use crate::components::button::Button;
use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::utils::api::sign_in::signin;
use dioxus::prelude::*;

use crate::components::error_line::OverallErrorLine;
use crate::components::input::InputField;
use crate::entities::input::UserInput;
use crate::state::AppState;
use crate::utils;

#[component]
pub fn SignInPage() -> Element {
    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);
    let mut in_progress = use_signal(|| false);
    let mut user_input = use_signal(UserInput::new);
    let mut app_context = consume_context::<Signal<AppState>>();

    let onsubmit = move |_: FormEvent| async move {
        error_message.set(None);
        in_progress.set(true);
        let entry = user_input.read();

        let response = signin(entry.get("username"), entry.get("password")).await;

        match response {
            Ok(_) => {
                utils::navigate_back_or_home();
            }
            Err(ApplicationError::BadRequestError(payload)) => {
                error_message.set(Some(payload));
            }
            Err(e) => {
                utils::handle_application_error(&mut app_context, e);
            }
        }
        in_progress.set(false);
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
                "Sign in to your account"
            }
        }
        div { class: "mt-10 sm:mx-auto sm:w-full sm:max-w-prose",
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
                div {
                    div { class: "flex items-center justify-between",
                        label {
                            r#for: "password",
                            class: "block text-sm font-medium leading-6 text-gray-900",
                            "Password"
                        }
                        div { class: "text-sm",
                            Link {
                                to: Route::InitiateResetPasswordPage {},
                                class: "font-semibold text-indigo-600 hover:text-indigo-500",
                                "Forgot password?"
                            }
                        }
                    }
                    div { class: "mt-2",
                        InputField {
                            required: "true",
                            autocomplete: "current-password",
                            error_payload: error_message,
                            identifier: "password",
                            typ: "password",
                            value: user_input.read().get("password"),
                            oninput: move |event: Event<FormData>| user_input.write().set("password", event.value())
                        }
                    }
                }
                div {
                    Button {
                        progress: *in_progress.read(),
                        r#type: "submit",
                        class: "flex w-full justify-center rounded-md bg-indigo-600 disabled:bg-neutral-600  h-10 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 mr-3 ",
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
