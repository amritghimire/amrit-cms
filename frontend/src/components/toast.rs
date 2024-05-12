use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn ToastMessageBox() -> Element {
    let app_context = consume_context::<Signal<AppState>>();

    rsx! {
        div {
            class: "absolute bottom-3 end-3 space-y-3",
            for toast in app_context.read().toast_messages.iter() {
                 div {
                    key: "{toast.id}",
                    role: "alert",
                    class: "{toast.class()}",
                    class: "max-w-xs rounded-xl text-sm text-white shadow-lg",
                    div { class: "flex p-4",
                        "{toast.message}"
                        ClearToastMessage {
                            id: toast.id
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ClearToastMessage(id: u16) -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();
    let onclick = move |_| {
        app_context
            .write()
            .toast_messages
            .retain(|msg| msg.id != id);
    };

    rsx! {
        div { class: "ms-auto",
            button {
                r#type: "button",
                onclick,
                class: "inline-flex size-5 flex-shrink-0 items-center justify-center rounded-lg text-white opacity-50 hover:text-white hover:opacity-100 focus:opacity-100 focus:outline-none",
                span { class: "sr-only", "Close" }
                svg {
                    "stroke-width": "2",
                    width: "24",
                    height: "24",
                    "xmlns": "http://www.w3.org/2000/svg",
                    "stroke-linecap": "round",
                    "fill": "none",
                    "viewBox": "0 0 24 24",
                    "stroke-linejoin": "round",
                    "stroke": "currentColor",
                    class: "size-4 flex-shrink-0",
                    path { "d": "M18 6 6 18" }
                    path { "d": "m6 6 12 12" }
                }
            }
        }
    }
}
