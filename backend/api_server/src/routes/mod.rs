use crate::handlers;
use axum::routing::method_routing::get;
use axum::routing::Router;

pub fn base_routes() -> Router {
    Router::new().route("/health_check", get(handlers::health_check))
}

pub fn create_router() -> Router {
    Router::new()
        .merge(base_routes())
        .nest(
            "/subscriptions",
            subscription_service::router::create_router(),
        )
        .fallback(handlers::not_found)
}
