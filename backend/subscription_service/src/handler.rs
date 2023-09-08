use crate::extractor::SubscriptionPayload;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sqlx::{Error, PgPool};
use sqlx::postgres::PgQueryResult;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;
use uuid::Uuid;

#[tracing::instrument(name="Subscription request",
skip(state, payload), fields(
email= %payload.email,
name= %payload.name
))]
pub async fn subscribe(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<SubscriptionPayload>,
) -> Response {
    let pool = &state.connection;
    tracing::info!("Adding a new subscription");
    match insert_subscriber(pool, &payload)
    .await
    {
        Ok(_) => {
            send_confirmation_link(&state, payload)
        }
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            handle_email_error(err).into_response()
        }
    }
}

#[tracing::instrument(
name = "Inserting subscriber to database",
skip(pool, payload)
)]
async fn insert_subscriber(pool: &PgPool, payload: &SubscriptionPayload) -> Result<PgQueryResult, Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        time::OffsetDateTime::now_utc()
    )
        .execute(pool)
        .await
}


fn handle_email_error(err: Error) -> ErrorPayload {
    if let Some(e) = err.into_database_error() {
        let message: &str = e.message();
        if message.contains("subscriptions_email_key")
            && message.contains("duplicate key value")
        {
            tracing::info!("Email already exists");
            return ErrorPayload::new("Email already subscribed", Some("error"), Some(400));
        }
    }

    ErrorPayload::new("Unable to add to subscription", Some("error"), Some(500))
}

#[tracing::instrument(
name = "Sending confirmation link",
skip(state, payload)
)]
fn send_confirmation_link(state: &AppState, payload: SubscriptionPayload) -> Response {
    let confirmation_link = format!("{}/subscription/confirm", state.settings.application.full_url());
    match state.email_client.to_owned().unwrap().send_email(
        payload.email,
        "Welcome to our newsletter!".to_string(),
        format!("Welcome to our newsletter. Please visit {} to confirm your subscription", { confirmation_link }),
    ) {
        Ok(_) => Json(json!({"ok": 1})).into_response(),
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            ErrorPayload::new("Unable to send email", Some("error"), Some(400))
                .into_response()
        }
    }
}
