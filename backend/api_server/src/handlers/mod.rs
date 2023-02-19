use axum::http::StatusCode;
use uuid::Uuid;

#[tracing::instrument(name="Health check request", fields(
    request_id = %Uuid::new_v4()
))]
pub async fn health_check() -> Result<String, StatusCode> {
    tracing::info!("Health check called");
    Ok("Ok".to_string())
}

pub async fn not_found() -> Result<String, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}
