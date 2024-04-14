use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::utils::api::sign_up::{signup, RegistrationPayload};
use dioxus::prelude::*;

use crate::components::error_line::OverallErrorLine;
use crate::components::input::InputField;
use crate::entities::input::UserInput;

#[component]
pub fn SignUpPage() -> Element {
    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);
    let mut user_input = use_signal(UserInput::new);

    let onsubmit = move |_: FormEvent| async move {
        let payload = RegistrationPayload::from(user_input);
        error_message.set(None);

        let response = signup(payload).await;
        if response.is_ok() {
            let nav = navigator();
            nav.replace(Route::Home {});
        }
        if let Err(ApplicationError::BadRequestError(payload)) = response {
            error_message.set(Some(payload));
        }
    };

    rsx! {
        div { class: "flex min-h-full flex-col justify-center px-6 py-12 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-prose",
            img {
                src: "https://tailwindui.com/img/logos/mark.svg?color=indigo&shade=600",
                alt: "AmritCMS",
                class: "mx-auto h-10 w-auto"
            }
            h2 { class: "mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900",
                "Create an account"
            }
        }
        div { class: "mt-2 sm:mx-auto sm:w-full sm:max-w-prose",
            form { onsubmit,
                OverallErrorLine {
                    error_payload: error_message
                }
                div {
                    div { class: "pb-4",
                        p { class: "mt-1 text-sm leading-6 text-gray-600",
                            "Please fill up the information below to create an account"
                        }
                        div { class: "mt-2 grid grid-cols-2 gap-x-6 gap-y-8 sm:grid-cols-6",
                            div { class: "col-span-full",
                                label {
                                    r#for: "username",
                                    class: "block text-sm font-medium leading-6 text-gray-900",
                                    "Username"
                                }
                                div { class: "mt-2",
                                        InputField {
                                            required: "true",
                                            autocomplete: "username",
                                            error_payload: error_message,
                                            identifier: "username",
                                            typ: "username",
                                            value: user_input.read().get("username"),
                                            oninput: move |event: Event<FormData>| user_input.write().set("username", event.value())
                                        }
                                }
                            }
                        }
                    }
                    div { class: "pb-4",
                        div { class: "mt-2 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6",
                            div { class: "sm:col-span-3",
                                label {
                                    r#for: "password",
                                    class: "block text-sm font-medium leading-6 text-gray-900",
                                    "Password"
                                }
                                div { class: "mt-2",
                                    InputField {
                                        required: "true",
                                        autocomplete: "new-password",
                                        error_payload: error_message,
                                        identifier: "password",
                                        typ: "password",
                                        value: user_input.read().get("password"),
                                        oninput: move |event: Event<FormData>| user_input.write().set("password", event.value())
                                    }
                                }
                            }
                            div { class: "sm:col-span-3",
                                label {
                                    r#for: "confirm_password",
                                    class: "block text-sm font-medium leading-6 text-gray-900",
                                    "Confirm Password"
                                }
                                div { class: "mt-2",
                                    InputField {
                                        required: "true",
                                        autocomplete: "new-password",
                                        error_payload: error_message,
                                        identifier: "confirm_password",
                                        typ: "password",
                                        value: user_input.read().get("confirm_password"),
                                        oninput: move |event: Event<FormData>| user_input.write().set("confirm_password", event.value())
                                    }
                                }
                            }
                            div { class: "col-span-full",
                                label {
                                    r#for: "email",
                                    class: "block text-sm font-medium leading-6 text-gray-900",
                                    "Email address"
                                }
                                div { class: "mt-2",
                                    InputField {
                                        required: "true",
                                        autocomplete: "email",
                                        error_payload: error_message,
                                        identifier: "email",
                                        typ: "email",
                                        value: user_input.read().get("email"),
                                        oninput: move |event: Event<FormData>| user_input.write().set("email", event.value())
                                    }
                                }
                            }
                            div { class: "col-span-full",
                                label {
                                    r#for: "name",
                                    class: "block text-sm font-medium leading-6 text-gray-900",
                                    "Full name"
                                }
                                div { class: "mt-2",
                                    InputField {
                                        required: "true",
                                        autocomplete: "name",
                                        error_payload: error_message,
                                        identifier: "name",
                                        typ: "text",
                                        value: user_input.read().get("name"),
                                        oninput: move |event: Event<FormData>| user_input.write().set("name", event.value())
                                    }
                                }
                            }
                        }
                    }
                    div { class: "pb-4",
                        div { class: "mt-2 space-y-10",
                            fieldset {
                                div { class: "mt-6 space-y-6",
                                    div { class: "relative flex gap-x-3",
                                        div { class: "flex h-6 items-center",
                                            input {
                                                r#type: "checkbox",
                                                name: "tos",
                                                class: "h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-600",
                                                id: "tos",
                                                required: "required"
                                            }
                                        }
                                        div { class: "text-sm leading-6",
                                            label {
                                                r#for: "tos",
                                                class: "font-medium text-gray-900",
                                                "Terms and conditions"
                                            }
                                            p { class: "text-gray-500",
                                                "I have read the terms and conditions"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "mt-6 flex items-center justify-end gap-x-6",
                    button {
                        r#type: "button",
                        class: "text-sm font-semibold leading-6 text-gray-900",
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
                        "Sign Up"
                    }
                }
            }
        }
    }
    }
}
