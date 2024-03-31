#![allow(non_snake_case)]

use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, StaticRouter};

use components::nav;

#[derive(Route, Clone, Copy)]
pub enum AppRoutes {
    #[to("/")]
    Index,
    #[not_found]
    NotFound,
}

fn switch<G: Html>(route: ReadSignal<AppRoutes>) -> View<G> {
    view! {
        div {
            nav::Navbar()
            (match route.get() {
                AppRoutes::Index => view! {
                   "Index page"
                },
                AppRoutes::NotFound => view! {
                    "404 Not Found"
                },
            })
        }
    }
}

/// # Props
/// * `pathname` - Set to `Some(_)` if running on the server.
#[component]
pub fn App<G: Html>(pathname: Option<String>) -> View<G> {
    match pathname {
        Some(pathname) => {
            let route = AppRoutes::default().match_path(&pathname);
            view! {
                StaticRouter(
                    view=switch,
                    route=route,
                )
            }
        }
        None => view! {
            Router(
                view=switch,
                integration=HistoryIntegration::new(),
            )
        },
    }
}
