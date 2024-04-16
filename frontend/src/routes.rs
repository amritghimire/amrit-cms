use dioxus::prelude::*;

use crate::pages::{
    AuthenticatedLayout, ConfirmationPage, Home, PageNotFound, SignInPage, SignUpPage,
};

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    #[layout(AuthenticatedLayout)]
    #[route("/")]
    Home {},
    #[route("/auth/confirm/:token")]
    ConfirmationPage { token: String },
    #[end_layout]
    #[route("/auth/login")]
    SignInPage {},
    #[route("/auth/signup")]
    SignUpPage {},
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
