use axum::http::StatusCode;

#[tracing::instrument(name = "Health check request")]
pub async fn health_check() -> Result<String, StatusCode> {
    tracing::info!("Health check called");
    Ok("Ok".to_string())
}

pub async fn not_found() -> Result<String, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}
