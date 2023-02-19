use crate::handlers;
use axum::routing::method_routing::get;
use axum::routing::Router;
use utils::configuration::Settings;
use utils::state::AppState;


pub fn base_routes() -> Router<AppState> {
    Router::new().route("/health_check", get(handlers::health_check))
}


pub async fn create_router() -> Router {
    let settings = Settings::new().expect("Failed to read configuration");
    let app_state = AppState::init(settings).await;

    Router::new()
        // .layer(sb)
        .merge(base_routes())
        .nest(
            "/subscriptions",
            subscription_service::router::create_router(),
        )
        .fallback(handlers::not_found)
        .with_state(app_state)
}
