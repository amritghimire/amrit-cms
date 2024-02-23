use crate::errors::user::UserError;
use crate::extractors::authentication::LoggedInUser;
use crate::helpers::sessions::delete_session;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use utils::errors::ErrorPayload;
use utils::state::AppState;

#[tracing::instrument(name = "getting current login", skip(user, state), fields(username = %user.user.username))]
pub async fn logout(
    user: LoggedInUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    let mut transaction = pool.begin().await.map_err(UserError::SessionError)?;
    delete_session(&mut transaction, user.session).await?;

    Ok(Json(json!({"ok": true})))
}
