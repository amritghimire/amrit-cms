use axum::routing::Router;
use utils::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
}
