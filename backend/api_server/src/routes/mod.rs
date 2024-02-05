use crate::handlers;
use axum::routing::method_routing::get;
use axum::routing::Router;
use tower::ServiceBuilder;
use tower_http::request_id::MakeRequestUuid;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use utils::configuration::Settings;
use utils::state::AppState;
use crate::apps::applications;

pub fn base_routes() -> Router<AppState> {
    Router::new().route("/health_check", get(handlers::health_check))
}

pub async fn create_router() -> Router {
    let settings = Settings::new().expect("Failed to read configuration");
    let app_state = AppState::init(settings).await;
    let apps = applications(&app_state.connection).await;

    let svc = ServiceBuilder::new()
        // make sure to set request ids before the request reaches `TraceLayer`
        .set_x_request_id(MakeRequestUuid)
        // log requests and responses
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        // propagate the header to the response before the response reaches `TraceLayer`
        .propagate_x_request_id();

    let mut router = Router::new().merge(base_routes());
    for app in apps {
        router = app.add_routes(router);
    }
    router.fallback(handlers::not_found)
        .with_state(app_state)
        .layer(svc)
}
