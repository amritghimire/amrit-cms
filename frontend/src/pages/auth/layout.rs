use crate::components::navbar::authenticated::AuthenticatedNavbar;
use crate::pages::auth::UserNotVerified;
use dioxus::prelude::*;
use log::info;

use crate::routes::Route;
use crate::state::AppState;
use crate::utils;
use crate::utils::api::me::me;

async fn handle_authentication(app_context: &mut Signal<AppState>) {
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
            utils::handle_application_error(app_context, e);
        }
    }
}

#[component]
pub fn AuthenticatedLayout() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();

    spawn(async move {
        handle_authentication(&mut app_context).await;
    });

    rsx! {
        if app_context().user.is_some() {
            AuthenticatedNavbar {}
        } else{
            main { class: "grid min-h-full place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
                div { class: "text-center",
                    p { class: "text-base font-semibold text-indigo-600", "..." }
                    h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                        "Checking for authentication...."
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
pub fn VerifiedLayout() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();

    spawn(async move {
        handle_authentication(&mut app_context).await;
    });

    rsx! {
        if app_context.read().user.as_ref().is_some_and(|u| { u.is_confirmed })  {
            AuthenticatedNavbar {}
            Outlet::<Route> {}
        } else if app_context.read().user.is_some() {
            AuthenticatedNavbar {}
            UserNotVerified {}
        } else {
            main { class: "grid min-h-full place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
                div { class: "text-center",
                    p { class: "text-base font-semibold text-indigo-600", "..." }
                    h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                            "Checking for authentication...."
                    }
                }
            }
        }

    }
}
