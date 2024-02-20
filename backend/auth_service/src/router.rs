use crate::handlers::confirmation::confirm;
use crate::handlers::registration::register;
use axum::routing::{get, post, Router};
use utils::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/confirm/:token", get(confirm))
}
