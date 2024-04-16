use crate::errors::ApplicationError;
use crate::routes::Route;
use crate::utils::api::confirm::confirm_token;
use dioxus::prelude::*;
use log::info;

#[component]
pub fn ConfirmationPage(token: String) -> Element {
    let token_signal = use_signal(|| token);

    let future = use_resource(move || async move {
        info!("Spawning task");

        let response = confirm_token(&token_signal.read()).await;
        if let Err(ApplicationError::BadRequestError(payload)) = response {
            let message = payload.message.clone();
            Some(message)
        } else {
            None
        }
    });

    rsx! {
        main { class: "grid min-h-full place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
            div { class: "text-center",
                p { class: "text-base font-semibold text-indigo-600", "..." }
                match &*future.read_unchecked() {
                    Some(Some(response)) => {
                        rsx! {
                            h1 {
                                class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl text-red-500",
                                "Confirmation failed."
                            }
                            p {
                                class: "mt-6 text-base leading-7 text-gray-600",
                                "{response}"
                            }
                        }
                    }
                    Some(None) => {
                        rsx! {
                            h1 {
                                class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                                "Successfully confirmed."
                            }
                            p {
                                class: "mt-6 text-base leading-7 text-gray-600",
                                "Your action has been successfully confirmed."
                            }
                        }
                    }
                    None => {
                        rsx! {
                            h1 {
                                class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                                "Confirming your action."
                            }
                            p {
                                class: "mt-6 text-base leading-7 text-gray-600",
                                "Please wait while we validate your request."
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
        }
    }
}
