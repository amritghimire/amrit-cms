use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::utils::api::sign_up::{signup, RegistrationPayload};
use dioxus::prelude::*;

use crate::components::error_line::{ErrorLine, OverallErrorLine};

#[component]
pub fn SignUpPage() -> Element {
    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);

    let onsubmit = move |evt: FormEvent| async move {
        let form_values = &evt.values();
        let payload = RegistrationPayload::from(form_values);
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
            div { class: "sm:mx-auto sm:w-full sm:max-w-sm",
            img {
                src: "https://tailwindui.com/img/logos/mark.svg?color=indigo&shade=600",
                alt: "AmritCMS",
                class: "mx-auto h-10 w-auto"
            }
            h2 { class: "mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900",
                "Create an account"
            }
        }
        div { class: "mt-2 sm:mx-auto sm:w-full sm:max-w-sm",
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
                                    div { class: "flex rounded-md shadow-sm ring-1 ring-inset ring-gray-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-600 sm:max-w-md",
                                        input {
                                            autocomplete: "username",
                                            r#type: "text",
                                            name: "username",
                                            placeholder: "username",
                                            class: "block flex-1 border-0 bg-transparent py-1.5 pl-1 text-gray-900 placeholder:text-gray-400 focus:ring-0 sm:text-sm sm:leading-6",
                                            id: "username"
                                        }
                                    }
                                    ErrorLine {
                                        field: "username",
                                        error_payload: error_message
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
                                    input {
                                        autocomplete: "new-password",
                                        r#type: "password",
                                        name: "password",
                                        class: "block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                                        id: "password"
                                    }
                                    ErrorLine {
                                        field: "password",
                                        error_payload: error_message
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
                                    input {
                                        name: "confirm_password",
                                        autocomplete: "new-password",
                                        r#type: "password",
                                        class: "block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                                        id: "confirm_password"
                                    }
                                    ErrorLine {
                                        field: "confirm_password",
                                        error_payload: error_message
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
                                    input {
                                        name: "email",
                                        r#type: "email",
                                        autocomplete: "email",
                                        class: "block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                                        id: "email"
                                    }
                                    ErrorLine {
                                        field: "email",
                                        error_payload: error_message
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
                                    input {
                                        r#type: "text",
                                        name: "name",
                                        autocomplete: "name",
                                        class: "block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                                        id: "name"
                                    }
                                    ErrorLine {
                                        field: "name",
                                        error_payload: error_message
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
