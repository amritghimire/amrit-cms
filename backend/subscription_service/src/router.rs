use axum::routing::method_routing::post;
use axum::routing::Router;

use crate::handler::subscribe;

pub fn create_router() -> Router {
    Router::new().route("/", post(subscribe))
}
