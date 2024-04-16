use crate::components::navbar::authenticated::AuthenticatedNavbar;
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
        if app_context().user.is_some() {
            AuthenticatedNavbar {}
            Outlet::<Route> {}
        } else{
            main { class: "grid min-h-full place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
                div { class: "text-center",
                    p { class: "text-base font-semibold text-indigo-600", "404" }
                    h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                        if error_message.read().is_empty() {
                            "Checking for authentication...."
                        } else {
                            "{error_message}"
                        }
                    }
                }
            }
        }
    }
}
