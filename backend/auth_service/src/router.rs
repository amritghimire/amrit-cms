use crate::handlers::confirmation::{confirm, resend_verification};
use crate::handlers::login::login;
use crate::handlers::logout::logout;
use crate::handlers::me::me;
use crate::handlers::registration::register;
use crate::handlers::reset::{check_reset_token, initiate_reset_password, reset_password};
use axum::routing::{get, post, Router};
use utils::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/resend-verification", post(resend_verification))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/confirm/:token", post(confirm))
        .route("/initiate-reset", post(initiate_reset_password))
        .route("/check-reset/:token", post(check_reset_token))
        .route("/reset-password/:token", post(reset_password))
}
