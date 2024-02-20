use crate::errors::auth::UserRegistrationError;
use crate::extractor::{Confirmation, ConfirmationActionType, User};
use crate::helpers::confirmation::{add_confirmation, send_verification_link};
use crate::helpers::user::insert_user;
use crate::payload::RegisterPayload;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;

#[tracing::instrument(name="Starting a registration",
skip(state, payload), fields(
name= %payload.name,
username= %payload.username,
email= %payload.email
)
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<RegisterPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    tracing::info!("Creating a new registration");
    let mut transaction = pool.begin().await.map_err(UserRegistrationError::Pool)?;

    User::check_acceptable_password(&payload.password, &[&payload.name, &payload.username])?;
    let mut user = User::try_from(payload)?;
    let id = insert_user(&mut transaction, &user).await?;
    user.id = id;
    let (confirmation, token) = Confirmation::new(
        user.id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );
    add_confirmation(&mut transaction, &confirmation).await?;

    transaction
        .commit()
        .await
        .map_err(UserRegistrationError::TransactionCommitError)?;

    send_verification_link(&state, &user, &confirmation, token).await?;
    Ok(Json(user))
}
