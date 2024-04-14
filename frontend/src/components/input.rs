use crate::components::error_line::ErrorLine;
use crate::errors::ErrorPayload;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct InputProps {
    error_payload: Signal<Option<ErrorPayload>>,
    #[props(into, default = "false")]
    required: &'static str,
    #[props(
        into,
        default = "block w-full rounded-md border-0 p-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
    )]
    class: &'static str,
    #[props(into)]
    autocomplete: String,
    #[props(into)]
    identifier: &'static str,
    #[props(into)]
    typ: String,
    #[props(into)]
    value: String,
    oninput: EventHandler<FormEvent>,
}

#[component]
pub fn InputField(props: InputProps) -> Element {
    rsx! {
        input {
            required: props.required,
            autocomplete: props.autocomplete,
            name: props.identifier,
            r#type: props.typ,
            class: props.class,
            id: props.identifier,
            value: props.value,
            oninput: move |event| props.oninput.call(event)
        }
        ErrorLine {
            field: props.identifier,
            error_payload: props.error_payload
        }
    }
}
