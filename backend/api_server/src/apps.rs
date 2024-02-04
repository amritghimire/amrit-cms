use axum::Router;
use sqlx::PgPool;
use utils::state::AppState;

#[macro_export(local_inner_macros)]
macro_rules! apps_internal {
    ($url: literal, $name: literal, true) => {
        sqlx::migrate!("../$name/migrations")
            .run(pool)
            .await
            .expect("Migrations failed :(");

        app_lists.push(
            AppConfig { url: $url, name: $name, router: $name::router::create_router }
        );
    };
    ($url: literal, $name: literal, false) => {
        app_lists.push(
            AppConfig { url: $url, name: $name, router: $name::router::create_router }
        );
    };

}

#[macro_export(local_inner_macros)]
macro_rules! apps {
    ($($app:tt)+) => {
        let mut app_lists:Vec<AppConfig> = vec![];
        apps_internal!($($app)+)
    };
}


pub struct AppConfig {
    url: &'static str,
    name: &'static str,
    router: fn() -> Router<AppState>
}


pub async fn migrate(pool: &PgPool) {
    // subscription_service::migrate(pool).await;


    sqlx::migrate!("../subscription_service/migrations")
        .run(pool)
        .await
        .expect("Migrations failed :(");
}

pub fn apps() -> Vec<AppConfig> {
    apps![
        ("/subscriptions", "subscription_service", true),
    ];
    let mut app_lists:Vec<AppConfig> = vec![];
    if true {
        app_lists.push(
            AppConfig { url: "/subscriptions", name: "subscription_service", router: subscription_service::router::create_router }
        );
    }

    vec![
        AppConfig { url: "/subscriptions", name: "subscription_service", router: subscription_service::router::create_router}
    ]
}

