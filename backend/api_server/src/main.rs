use api_server::routes;
use once_cell::sync::Lazy;
use sqlx::PgPool;

static TRACING: Lazy<()> = Lazy::new(|| {});

#[shuttle_runtime::main]
pub async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    subscription_service::migrate(&pool).await;

    Lazy::force(&TRACING);

    let router = routes::create_router(pool).await;
    Ok(router.into())
}
