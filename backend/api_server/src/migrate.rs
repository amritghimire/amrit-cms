use sqlx::PgPool;
use crate::apps::applications;

pub async fn migrate_all_apps(pool: &PgPool) {
    applications(pool).await;
}