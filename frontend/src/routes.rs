use dioxus::prelude::*;

use crate::pages::{AuthenticatedLayout, Home, PageNotFound};

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    #[layout(AuthenticatedLayout)]
    #[route("/")]
    Home {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
