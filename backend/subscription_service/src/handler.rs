use crate::extractor::SubscriptionPayload;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
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
    let pool = state.connection;
    tracing::info!("Adding a new subscription");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        time::OffsetDateTime::now_utc()
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {
            match state.email_client.unwrap().send_email(
                payload.email,
                "Welcome to our newsletter!".to_string(),
                "Hey".to_string(),
            ) {
                Ok(_) => Json(json!({"ok": 1})).into_response(),
                Err(err) => {
                    tracing::error!("Error occurred {:?}", err);
                    ErrorPayload::new("Unable to send email", Some("error"), Some(400))
                        .into_response()
                }
            }
        }
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            if let Some(e) = err.into_database_error() {
                let message: &str = e.message();
                if message.contains("subscriptions_email_key")
                    && message.contains("duplicate key value")
                {
                    tracing::info!("Email already exists");
                    return ErrorPayload::new("Email already subscribed", Some("error"), Some(400))
                        .into_response();
                }
            }

            ErrorPayload::new("Unable to add to subscription", Some("error"), Some(500))
                .into_response()
        }
    }
}
