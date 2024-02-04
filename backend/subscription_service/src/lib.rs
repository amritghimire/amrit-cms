extern crate util_macros;

use sqlx::PgPool;

pub mod errors;
mod extractor;
mod handler;
pub mod helper;
pub mod router;

pub async fn migrate(pool: &PgPool){
    sqlx::migrate!()
        .run(pool)
        .await
        .expect("Migrations failed :(");
}