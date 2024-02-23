use crate::extractors::authentication::LoggedInUser;
use axum::response::IntoResponse;
use axum::Json;

#[tracing::instrument(name = "getting current login", skip(user), fields(username = %user.user.username))]
pub async fn me(user: LoggedInUser) -> impl IntoResponse {
    Json(user.user)
}
