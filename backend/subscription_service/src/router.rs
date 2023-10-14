use axum::routing::method_routing::post;
use axum::routing::{get, Router};
use utils::state::AppState;

use crate::handler::{confirm, subscribe};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/confirm", get(confirm))
        .route("/", post(subscribe))
}
