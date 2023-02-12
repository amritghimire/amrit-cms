use crate::extractor::SubscriptionPayload;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use utils::errors::ErrorPayload;
use utils::validation::ValidatedForm;
use uuid::Uuid;

pub async fn subscribe(
    State(pool): State<PgPool>,
    ValidatedForm(payload): ValidatedForm<SubscriptionPayload>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        Utc::now()
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Json(json!({"ok": 1})).into_response(),
        Err(_) => ErrorPayload::new("Unable to add to subscription", Some("error"), Some(400))
            .into_response(),
    }
}
