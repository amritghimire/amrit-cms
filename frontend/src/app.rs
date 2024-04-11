use crate::errors::ApplicationError;
use crate::routes::Route;
use crate::state::AppState;
use crate::utils;
use dioxus::dioxus_core::CapturedError;
use dioxus::prelude::*;

fn handle_error(error: CapturedError) -> Element {
    if let Some(ApplicationError::Unauthorized) = error.downcast::<ApplicationError>() {
        utils::redirect_to_login();
    }
    rsx! {
        "Hmm, something went wrong. Please report {error} to the developer of this application"
    }
}

pub fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        ErrorBoundary {
            handle_error: |error| {
                handle_error(error)
            },
            Router::<Route> {}
        }
    }
}
