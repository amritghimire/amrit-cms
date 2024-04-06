use dioxus::prelude::*;
use log::info;

use crate::routes::Route;
use crate::state::AppState;
use crate::utils::api::me;

#[component]
pub fn AuthenticatedLayout() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();
    spawn(async move {
        info!("Spawning task");
        let response = me().await;
        match response {
            Ok(user) => {
                app_context.write().user = Some(user);
            }
            Err(_) => {
                // Redirect to log in
                info!("Redirecting back to login");
                web_sys::window()
                    .unwrap()
                    .location()
                    .set_href("/auth/login")
                    .unwrap();
            }
        };
    });

    if app_context().user.is_some() {
        rsx! {
            h1 { "user is authenticated"}
            Outlet::<Route> {}
        }
    } else {
        rsx! {
            h1 { "Loading auth level........"}
            Outlet::<Route> {}

        }
    }
}
