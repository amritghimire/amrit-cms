use crate::routes::Route;
use crate::state::AppState;
use dioxus::prelude::*;

pub fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        Router::<Route> {}
    }
}
