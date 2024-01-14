use axum::routing::method_routing::post;
use axum::routing::{get, Router};
use utils::state::AppState;

use crate::handler::{confirm, publish_newsletter, subscribe};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/newsletter", post(publish_newsletter))
        .route("/confirm", get(confirm))
        .route("/", post(subscribe))
}
