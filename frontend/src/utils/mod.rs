use crate::routes::Route;
use dioxus::prelude::navigator;
use log::info;

pub(crate) mod api;

pub fn redirect_to_login() {
    info!("Redirecting back to login");
    let nav = navigator();
    nav.replace(Route::SignInPage {});
}
