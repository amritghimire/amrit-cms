use crate::extractor::SubscriptionPayload;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;
use utils::errors::ErrorPayload;
use utils::validation::ValidatedForm;
use uuid::Uuid;

#[tracing::instrument(name="Subscription request",
skip(pool, payload), fields(
email= %payload.email,
name= %payload.name
))]
pub async fn subscribe(
    State(pool): State<PgPool>,
    ValidatedForm(payload): ValidatedForm<SubscriptionPayload>,
) -> impl IntoResponse {
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
        Ok(_) => Json(json!({"ok": 1})).into_response(),
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            ErrorPayload::new("Unable to add to subscription", Some("error"), Some(500))
                .into_response()
        }
    }
}
