use crate::routes::Route;
use dioxus::prelude::navigator;
use log::info;

pub(crate) mod api;

pub fn redirect_to_login() {
    info!("Redirecting back to login");
    let nav = navigator();
    nav.push(Route::SignInPage {});
}

pub fn navigate_back_or_home() {
    let nav = navigator();
    if nav.can_go_back() {
        nav.go_back();
    } else {
        nav.replace(Route::Home {});
    }
}
