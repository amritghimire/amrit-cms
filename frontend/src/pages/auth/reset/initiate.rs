use crate::components::button::Button;
use crate::components::error_line::OverallErrorLine;
use crate::components::input::InputField;
use crate::entities::input::UserInput;
use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::state::AppState;
use crate::utils;
use crate::utils::api::reset::initiate_reset;
use dioxus::prelude::*;

#[component]
pub fn InitiateResetPasswordPage() -> Element {
    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);
    let mut in_progress = use_signal(|| false);
    let mut sent = use_signal(|| false);
    let mut user_input = use_signal(UserInput::new);
    let mut app_context = consume_context::<Signal<AppState>>();

    let onsubmit = move |_: FormEvent| async move {
        error_message.set(None);
        in_progress.set(true);
        let entry = user_input.read();

        let response = initiate_reset(entry.get("username_or_email")).await;

        match response {
            Ok(_) => {
                sent.set(true);
            }
            Err(ApplicationError::BadRequestError(payload)) => {
                error_message.set(Some(payload));
            }
            Err(e) => {
                utils::handle_application_error(&mut app_context, e);
            }
        };
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
                "Reset your password"
            }
        }
        if *sent.read() {
            section { class: "relative isolate overflow-hidden bg-white px-6 py-24 sm:py-32 lg:px-8",
                div { class: "absolute inset-0 -z-10 bg-[radial-gradient(45rem_50rem_at_top,theme(colors.indigo.100),white)] opacity-20" }
                div { class: "absolute inset-y-0 right-1/2 -z-10 mr-16 w-[200%] origin-bottom-left skew-x-[-30deg] bg-white shadow-xl shadow-indigo-600/10 ring-1 ring-indigo-50 sm:mr-28 lg:mr-0 xl:mr-16 xl:origin-center" }
                div { class: "mx-auto max-w-2xl lg:max-w-4xl",
                    figure { class: "mt-10",
                        blockquote { class: "text-center text-xl font-semibold leading-8 text-gray-900 sm:text-2xl sm:leading-9",
                            p {
                                "If we have the email or username registered with us, we will send an email with reset link. Please check your email. If you haven't receive the email, please check the spam email."
                            }
                        }
                    }
                }
                div { class: "mt-10 flex items-center justify-center gap-x-6",
                    Link {
                        to: Route::Home {},
                        class: "rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
                        "Go back home"
                    }
                }
            }
        } else {
            div { class: "mt-10 sm:mx-auto sm:w-full sm:max-w-prose",
                form { onsubmit, class: "space-y-6",
                    OverallErrorLine {
                        error_payload: error_message
                    }
                    div {
                        label {
                            r#for: "username",
                            class: "block text-sm font-medium leading-6 text-gray-900",
                            "Username or email"
                        }
                        div { class: "mt-2",
                            InputField {
                                required: "true",
                                autocomplete: "username",
                                error_payload: error_message,
                                identifier: "username_or_email",
                                typ: "username",
                                value: user_input.read().get("username_or_email"),
                                oninput: move |event: Event<FormData>| user_input.write().set("username_or_email", event.value())
                            }
                        }
                   }
                    div {
                        Button {
                            progress: *in_progress.read(),
                            r#type: "submit",
                            class: "flex w-full justify-center rounded-md bg-indigo-600 disabled:bg-neutral-600  h-10 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 mr-3 ",
                            "Send reset email"
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
}
