mod client;

use crate::apps::applications;
use crate::handlers;
use axum::handler::HandlerWithoutStateExt;
use axum::routing::method_routing::get;
use axum::routing::Router;

use crate::routes::client::serve_frontend;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::request_id::MakeRequestUuid;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use utils::state::AppState;

pub fn base_routes() -> Router<AppState> {
    Router::new().route("/health_check", get(handlers::health_check))
}

pub async fn create_router(pool: PgPool) -> Router {
    let app_state = AppState::from_conn(pool);
    let serve_dir_path = app_state.settings.frontend.assets.clone();
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

    let mut api_router = Router::new().merge(base_routes());
    for app in apps {
        api_router = app.add_routes(api_router);
    }

    tracing::info!(
        "{}/index.html from {:?}",
        &serve_dir_path,
        std::env::current_dir()
    );

    let router = Router::new()
        .nest("/api", api_router.fallback(handlers::not_found))
        .fallback_service(ServeDir::new(serve_dir_path).fallback(serve_frontend.into_service()));
    router.with_state(app_state).layer(svc)
}
