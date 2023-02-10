use crate::extractor::SubscriptionPayload;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use utils::validation::ValidatedForm;

pub async fn subscribe(
    ValidatedForm(_payload): ValidatedForm<SubscriptionPayload>,
) -> impl IntoResponse {
    Json(json!({"ok": 1})).into_response()
}
