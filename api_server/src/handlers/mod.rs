use axum::http::StatusCode;

pub async fn health_check() -> Result<String, StatusCode> {
    Ok("Ok".to_string())
}

pub async fn not_found() -> Result<String, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}
