use crate::errors::auth::UserRegistrationError;
use crate::errors::confirm::ConfirmUserError;
use crate::errors::user::UserError;
use crate::extractors::confirmation::{Confirmation, ConfirmationActionType};
use crate::extractors::session::SESSION_TOKEN_COOKIE;
use crate::extractors::user::User;
use crate::helpers::confirmation::{
    add_confirmation, clear_confirmation_action_type, send_verification_link,
};
use crate::helpers::user::{fetch_by_email, fetch_by_username, fetch_user, update_password};
use axum::extract::{Path, State};
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use email_clients::email::{EmailAddress, EmailObject};
use secrecy::Secret;
use serde::Deserialize;
use serde_json::json;
use tokio::task;
use utils::errors::ErrorPayload;
use utils::state::{AppState, BackgroundTask};
use utils::validation::ValidatedForm;
use validator::Validate;

use crate::helpers::confirmation;
use crate::helpers::sessions::{clear_sessions, create_new_session};

#[derive(Debug, Deserialize, Validate)]
pub struct InitiateResetPasswordPayload {
    #[validate(length(min = 3, max = 255))]
    pub username_or_email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordPayload {
    #[validate(length(min = 8, max = 72, message = "Password must contains 8-72 characters"))]
    pub password: String,
    #[validate(must_match(
        other = "password",
        message = "password and confirm password must match"
    ))]
    pub confirm_password: String,
}

pub async fn initiate_reset_password(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<InitiateResetPasswordPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    let tx = state.tasks.clone();

    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;
    let user = if payload.username_or_email.contains('@') {
        fetch_by_email(&mut transaction, &payload.username_or_email)
            .await
            .map_err(UserRegistrationError::EmailCheckError)?
    } else {
        fetch_by_username(&mut transaction, &payload.username_or_email)
            .await
            .map_err(UserRegistrationError::UsernameCheck)?
    };

    tracing::info!("Starting password reset");
    let handle = task::spawn(async move {
        if let Some(user) = user {
            let (confirmation, confirmation_token) =
                Confirmation::new(user.id, ConfirmationActionType::PasswordReset, json!({}));

            add_confirmation(&mut transaction, &confirmation)
                .await
                .expect("Unable to add confirmation");
            send_verification_link(&state, &user, &confirmation, confirmation_token)
                .await
                .expect("Failed to send verification email");
            transaction
                .commit()
                .await
                .map_err(UserRegistrationError::TransactionCommitError)
                .expect("Failed to commit the transaction");
        }
    });
    if let Some(tx) = tx {
        let _ = tx.send(BackgroundTask::new("send_reset_password", handle));
    }

    Ok(Json(json!({})))
}

pub async fn check_reset_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    tracing::info!("starting confirmation token");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    let confirmation = confirmation::check_confirmation(token, &mut transaction).await?;
    if let ConfirmationActionType::PasswordReset = confirmation.action_type {
        let user_id = confirmation.user_id;
        let user = fetch_user(&mut transaction, user_id).await?;
        Ok(Json(json!({
            "name": user.name,
            "username": user.username
        })))
    } else {
        Err(ConfirmUserError::InvalidActionType)?
    }
}

pub async fn reset_password(
    jar: SignedCookieJar,
    State(state): State<AppState>,
    Path(token): Path<String>,
    ValidatedForm(payload): ValidatedForm<ResetPasswordPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    tracing::info!("starting confirmation token");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    let confirmation = confirmation::check_confirmation(token, &mut transaction).await?;

    if let ConfirmationActionType::PasswordReset = confirmation.action_type {
        let user_id = confirmation.user_id;
        let user = fetch_user(&mut transaction, user_id).await?;
        User::check_acceptable_password(&payload.password, &[&user.name, &user.username])?;
        let password = Secret::from(payload.password);
        let count = update_password(&mut transaction, user_id, password).await?;
        if count < 1 {
            Err(UserError::UnexpectedError)?;
        }
        clear_sessions(&mut transaction, user.id).await?;
        clear_confirmation_action_type(
            &mut transaction,
            user.id,
            ConfirmationActionType::PasswordReset,
        )
        .await?;
        let session_token = create_new_session(&mut transaction, user.id, json!({})).await?;
        transaction
            .commit()
            .await
            .map_err(UserRegistrationError::TransactionCommitError)?;

        let client = state.email_client.to_owned().unwrap();
        let email_object = EmailObject {
            sender: client.get_sender(),
            to: vec![EmailAddress {
                name: user.name.clone(),
                email: user.email.clone(),
            }],
            subject: "Your password was reset recently".to_string(),
            plain: format!(
                "Your password was successfully reset recently using your email address \
            for username: {}. If you didn't perform the reset, please change your password back \
            using the reset form and make sure your email is not compromised.",
                user.email
            ),
            html: format!(
                "Your password was successfully reset recently using your email address \
            for username: <b>{}</b>. If you didn't perform the reset, please change your password \
            back using the reset form and make sure your email is not compromised.",
                user.email
            ),
        };
        let handle = task::spawn(async move {
            client
                .send_emails(email_object)
                .await
                .map_err(UserError::ConfirmationEmailError)
                .expect("Unable to send email");
        });
        if let Some(tx) = &state.tasks {
            let _ = tx.send(BackgroundTask::new("send_confirmation_email", handle));
        }

        let session_header =
            HeaderValue::from_str(&session_token).map_err(UserRegistrationError::HeaderError)?;

        let jar = jar.add(Cookie::new(SESSION_TOKEN_COOKIE, session_token));
        let mut response = (jar, Json(user)).into_response();
        response.headers_mut().insert(AUTHORIZATION, session_header);

        Ok(response)
    } else {
        Err(ConfirmUserError::InvalidActionType)?
    }
}
