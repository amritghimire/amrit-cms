use axum::Router;
use sqlx::PgPool;
use utils::state::AppState;

use crate::installed_apps;
use crate::single_app;

installed_apps! {
    ("/subscriptions", subscription_service, "../subscription_service/migrations"),
    ("/auth", auth_service, "../auth_service/migrations")
}
