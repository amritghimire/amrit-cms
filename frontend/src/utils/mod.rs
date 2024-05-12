use crate::entities::toast::ToastType;
use crate::errors::ApplicationError;
use crate::routes::Route;
use crate::state::AppState;
use dioxus::prelude::{navigator, Signal, Writable};
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

pub fn handle_application_error(app_context: &mut Signal<AppState>, error: ApplicationError) {
    if let ApplicationError::Unauthorized = error {
        redirect_to_login();
    } else {
        app_context
            .write()
            .add_toast(ToastType::Error, error.to_string());
    }
}
