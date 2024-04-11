use crate::errors::ApplicationError;
use dioxus::prelude::*;
use log::info;

use crate::routes::Route;
use crate::state::AppState;
use crate::utils::api::me::me;
use crate::utils::redirect_to_login;

#[component]
pub fn AuthenticatedLayout() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();
    let mut error_message = use_signal(|| "".to_string());

    spawn(async move {
        info!("Spawning task");
        if app_context.read().user.is_some() {
            return;
        }
        let response = me().await;
        match response {
            Ok(user) => {
                app_context.write().user = Some(user);
            }
            Err(e) => {
                if let ApplicationError::Unauthorized = e {
                    redirect_to_login();
                } else {
                    error_message.set(e.to_string());
                }
            }
        };
    });

    rsx! {
        h1 {
            if app_context().user.is_some() {
                "Authenticated user"
            } else {
                "Loading auth level"
            }
        }
        h2 {
            "{error_message}"
        }
        Outlet::<Route> {}
    }
}
