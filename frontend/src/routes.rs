use dioxus::prelude::*;

use crate::pages::{
    AuthenticatedLayout, ConfirmationPage, Home, PageNotFound, SignInPage, SignUpPage,
    VerifiedLayout,
};

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    // Authenticated but verification not required.
    #[layout(AuthenticatedLayout)]
    #[route("/auth/confirm/:token")]
    ConfirmationPage { token: String },
    #[end_layout]
    // Verification required
    #[layout(VerifiedLayout)]
    #[route("/")]
    Home {},
    #[end_layout]
    // Out of authentication boundary
    #[route("/auth/login")]
    SignInPage {},
    #[route("/auth/signup")]
    SignUpPage {},
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
