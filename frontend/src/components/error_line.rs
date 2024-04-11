use crate::errors::ErrorPayload;
use dioxus::prelude::*;

#[component]
pub fn ErrorLine(field: String, error_payload: ReadOnlySignal<Option<ErrorPayload>>) -> Element {
    let error_message = use_memo(move || {
        error_payload
            .read()
            .as_ref()
            .map_or("".to_string(), |payload| payload.error_for_field(&field))
    });

    rsx! {
        if !error_message.read().is_empty()  {
            p {
                class: "mt-2 text-sm text-red-600 dark:text-red-500",
                span {
                    class: "font-medium",
                    "{error_message}"
                }
            }
        }
    }
}

#[component]
pub fn OverallErrorLine(error_payload: ReadOnlySignal<Option<ErrorPayload>>) -> Element {
    rsx! {
        if error_payload.read().is_some() {
            p {
                class: "mt-2 text-sm text-red-600 dark:text-red-500",
                span {
                    class: "font-medium",
                    "{error_payload.unwrap().message}"
                }
            }
        }
    }
}
