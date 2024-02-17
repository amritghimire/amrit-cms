#[macro_export]
macro_rules! installed_apps {
    ($($app:tt),*) => {
        pub struct AppConfig {
            url: &'static str,
            router: fn() -> Router<AppState>,
        }

        impl AppConfig {
            pub fn add_routes(self, mut router: Router<AppState>) -> Router<AppState> {
                router = router.nest(
                    self.url,
                    (self.router)()
                );
                router
            }
        }

        pub async fn applications(pool: &PgPool) -> Vec<AppConfig> {
            let mut app_lists: Vec<AppConfig> = Vec::new();
            $(
                single_app!($app, pool, &mut app_lists);
            )*
            app_lists
        }
    }
}

#[macro_export]
macro_rules! single_app {
    (($url:literal, $name:ident, $migrations:literal), $pool:expr, $app_lists:expr) => {
        let migrator = {
            let mut m = sqlx::migrate!($migrations);
            m.set_ignore_missing(true);
            // m.set_migrations_table(format!("_sqlx_migrations_{}", stringify!($name)));
            m
        };
        migrator.run($pool).await.expect("Migrations failed :(");
        $app_lists.push(AppConfig {
            url: $url,
            router: $name::router::create_router,
        });
    };
    (($url:literal, $name:ident), $pool:expr, $app_lists:expr) => {
        $app_lists.push(AppConfig {
            url: $url,
            router: $name::router::create_router,
        });
    };
}
