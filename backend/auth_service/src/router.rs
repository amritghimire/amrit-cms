use crate::handlers::confirmation::confirm;
use crate::handlers::login::login;
use crate::handlers::logout::logout;
use crate::handlers::me::me;
use crate::handlers::registration::register;
use axum::routing::{get, post, Router};
use utils::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/confirm/:token", post(confirm))
}
