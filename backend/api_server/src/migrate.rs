use crate::apps::applications;
use sqlx::PgPool;

pub async fn migrate_all_apps(pool: &PgPool) {
    applications(pool).await;
}
