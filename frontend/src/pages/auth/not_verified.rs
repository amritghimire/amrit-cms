use crate::components::button::Button;
use crate::entities::toast::ToastType;
use crate::state::AppState;
use crate::utils;
use crate::utils::api::confirm::resend_verification;
use dioxus::prelude::*;

#[component]
pub fn UserNotVerified() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();
    let mut in_progress = use_signal(|| false);

    let mut sent = use_signal(|| false);

    let onclick = move |_: MouseEvent| async move {
        in_progress.set(true);

        let response = resend_verification().await;
        match response {
            Ok(_) => {
                sent.set(true);
                app_context
                    .write()
                    .add_toast(ToastType::Info, "Verification email sent successfully.");
            }
            Err(e) => {
                utils::handle_application_error(&mut app_context, e);
            }
        }
        in_progress.set(false);
    };
    rsx! {
        main { class: "grid min-h-full place-items-center bg-white px-6 py-24 sm:py-32 lg:px-8",
            div { class: "text-center",
                p { class: "text-base font-semibold text-indigo-600", "Verification required" }
                h1 { class: "mt-4 text-3xl font-bold tracking-tight text-gray-900 sm:text-5xl",
                    if *sent.read() {
                        "Verification email sent."
                    } else {
                        "User email not verified"
                    }
                }
                p { class: "mt-6 text-base leading-7 text-gray-600",
                    "You need to verify your email address to proceed further. Please check your email to verify your account. "
                    br {}
                    if *sent.read() {
                        br {}
                        b {
                            "A new verification email is sent, please check."
                        }
                    } else {
                        "If you don't find your email, resend the verification email below."
                    }
                }
                if !*sent.read(){
                    div {
                        onclick,
                        class: "mt-10 flex items-center justify-center w-auto",
                        Button {
                            progress: *in_progress.read(),
                            class: "flex rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold disabled:bg-neutral-600  text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 ",
                            "Resend verification email"
                        }
                    }
                }
            }
        }
    }
}
