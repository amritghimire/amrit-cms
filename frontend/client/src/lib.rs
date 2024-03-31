#![allow(non_snake_case)]

use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, StaticRouter};

#[derive(Route, Clone)]
pub enum AppRoutes {
    #[to("/")]
    Index,
    #[not_found]
    NotFound,
}

fn switch<'a, G: Html>(cx: Scope<'a>, route: &'a ReadSignal<AppRoutes>) -> View<G> {
    view! { cx,
        div(class="text-lg") {
            (match route.get().as_ref() {
                AppRoutes::Index => view! { cx,
                   "Index page"
                },
                AppRoutes::NotFound => view! { cx,
                    "404 Not Found"
                },
            })
        }
    }
}

/// # Props
/// * `pathname` - Set to `Some(_)` if running on the server.
#[component]
pub fn App<G: Html>(cx: Scope, pathname: Option<String>) -> View<G> {
    match pathname {
        Some(pathname) => {
            let route = AppRoutes::default().match_path(&pathname);
            view! { cx,
                StaticRouter(
                    view=switch,
                    route=route,
                )
            }
        }
        None => view! { cx,
            Router(
                view=switch,
                integration=HistoryIntegration::new(),
            )
        },
    }
}
