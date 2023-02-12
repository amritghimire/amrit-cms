use axum::routing::method_routing::post;
use axum::routing::Router;
use utils::state::AppState;

use crate::handler::subscribe;

pub fn create_router() -> Router<AppState> {
    Router::new().route("/", post(subscribe))
}
