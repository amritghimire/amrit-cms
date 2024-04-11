use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::utils::api::sign_in::signin;
use dioxus::prelude::*;

use crate::components::error_line::{ErrorLine, OverallErrorLine};

#[component]
pub fn SignInPage() -> Element {
    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);

    let onsubmit = move |evt: FormEvent| async move {
        let username = &evt.values()["username"];
        let password = &evt.values()["password"];
        error_message.set(None);

        let response = signin(username.as_value(), password.as_value()).await;
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
                "Sign in to your account"
            }
        }
        div { class: "mt-10 sm:mx-auto sm:w-full sm:max-w-sm",
            form { onsubmit, class: "space-y-6",
                OverallErrorLine {
                    error_payload: error_message
                }
                div {
                    label {
                        r#for: "username",
                        class: "block text-sm font-medium leading-6 text-gray-900",
                        "Username"
                    }
                    div { class: "mt-2",
                        input {
                            required: "false",
                            autocomplete: "username",
                            name: "username",
                            r#type: "username",
                            class: "block w-full rounded-md border-0 p-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                            id: "username"
                        }
                        ErrorLine {
                            field: "username",
                            error_payload: error_message
                        }
                    }
               }
                div {
                    div { class: "flex items-center justify-between",
                        label {
                            r#for: "password",
                            class: "block text-sm font-medium leading-6 text-gray-900",
                            "Password"
                        }
                        div { class: "text-sm",
                            a {
                                href: "#",
                                class: "font-semibold text-indigo-600 hover:text-indigo-500",
                                "Forgot password?"
                            }
                        }
                    }
                    div { class: "mt-2",
                        input {
                            name: "password",
                            r#type: "password",
                            autocomplete: "current-password",
                            required: "false",
                            class: "block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6",
                            id: "password"
                        }
                        ErrorLine {
                            field: "password",
                            error_payload: error_message
                        }
                    }
                }
                div {
                    button {
                        r#type: "submit",
                        class: "flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
                        "Sign in"
                    }
                }
            }
            p { class: "mt-10 text-center text-sm text-gray-500",
                "\n      Not a member?\n      "
                Link {
                    to: Route::SignUpPage {},
                    class: "font-semibold leading-6 text-indigo-600 hover:text-indigo-500",
                    "Sign up now."
                }
            }
        }
    }
    }
}
