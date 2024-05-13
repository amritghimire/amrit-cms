use crate::components::button::Button;
use crate::components::error_line::OverallErrorLine;
use crate::components::input::InputField;
use crate::entities::input::UserInput;
use crate::entities::toast::ToastType;
use crate::errors::{ApplicationError, ErrorPayload};
use crate::routes::Route;
use crate::state::AppState;
use crate::utils;
use crate::utils::api::reset::{check_reset_token, reset_password, CheckResetTokenResponse};
use dioxus::prelude::*;
use log::info;

#[derive(PartialEq)]
enum PageState {
    Loading,
    TokenVerified(CheckResetTokenResponse),
    InvalidToken,
}

#[component]
pub fn ProcessResetLinkPage(token: String) -> Element {
    let token_signal = use_signal(|| token);
    let mut page_state = use_signal(|| PageState::Loading);

    let mut error_message: Signal<Option<ErrorPayload>> = use_signal(|| None);
    let mut in_progress = use_signal(|| false);
    let mut user_input = use_signal(UserInput::new);
    let mut app_context = consume_context::<Signal<AppState>>();

    spawn(async move {
        info!("Checking reset link");
        if PageState::Loading != *page_state.read() {
            return;
        }
        let result = check_reset_token((*token_signal.read()).to_string()).await;

        match result {
            Ok(response) => {
                page_state.set(PageState::TokenVerified(response));
            }
            Err(ApplicationError::BadRequestError(_)) => {
                page_state.set(PageState::InvalidToken);
            }
            Err(e) => {
                utils::handle_application_error(&mut app_context, e);
            }
        }
    });

    let onsubmit = move |_: FormEvent| async move {
        error_message.set(None);
        in_progress.set(true);
        let entry = user_input.read();

        let response = reset_password(
            &token_signal.read(),
            entry.get("password"),
            entry.get("confirm_password"),
        )
        .await;

        match response {
            Ok(_) => {
                app_context
                    .write()
                    .add_toast(ToastType::Success, "Password successfully reset");
                let nav = navigator();
                nav.replace(Route::SignInPage {});
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
        div { class: "flex flex-col justify-center px-6 py-12 lg:px-8",
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
        }
        match &(*page_state.read()) {
            PageState::Loading => {
                rsx! {
                    main { class: "grid place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
                        div { class: "text-center",
                            p { class: "text-base font-semibold text-indigo-600", "..." }
                            h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                                "Checking for validity of your token...."
                            }
                        }
                    }
                }
            }
            PageState::TokenVerified(user_info) => {
                rsx! {
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
                                            readonly: "true",
                                            autocomplete: "username",
                                            error_payload: error_message,
                                            identifier: "username",
                                            typ: "username",
                                            value: user_info.username.clone(),
                                            oninput: move |_| {}
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
                                }
                            }
                        }
                        div { class: "mt-6 flex items-center justify-end gap-x-6",
                            button {
                                r#type: "reset",
                                class: "text-sm font-semibold leading-6 text-gray-900",
                                "Cancel"
                            }
                            Button {
                                r#type: "submit",
                                progress: *in_progress.read(),
                                class: "rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
                                "Change password"
                            }
                        }
                    }
                }
                }
            }
            PageState::InvalidToken => {
                rsx! {
                    main { class: "grid place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
                        div { class: "text-center",
                            p { class: "text-base font-semibold text-indigo-600", "..." }
                            h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                                "The reset link is expired or invalid. Please try again."
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
                }
            }
        }
    }
}
