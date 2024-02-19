use crate::handlers::registration::register;
use axum::routing::{post, Router};
use utils::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new().route("/register", post(register))
}
