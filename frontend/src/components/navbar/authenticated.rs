use crate::state::AppState;
use crate::utils;
use crate::utils::api::logout::logout;
use dioxus::prelude::*;

#[component]
pub fn AuthenticatedNavbar() -> Element {
    let mut app_context = consume_context::<Signal<AppState>>();

    let onclick = move |_| async move {
        let response = logout().await;
        if response.is_ok() {
            app_context.write().user = None;

            utils::redirect_to_login();
        }
        // if let Err(ApplicationError::BadRequestError(_)) = response {
        //     // Couldn't logout. Add toast support here.
        // }
    };

    rsx! {
        header { class: "relative z-40 -mb-10 h-24 w-full sm:mb-0",
            div { class: "absolute inset-0 bg-white bg-opacity-90 backdrop-blur" }
            div { class: "relative mx-auto flex h-full flex-row items-center justify-between gap-4 px-6 xl:container",
                div { class: "flex h-full flex-nowrap items-center gap-20 2xl:gap-32",
                    a {
                        title: "AmritCMS",
                        href: "/en",
                        translate: "no",
                        class: "text-heading border-heading whitespace-nowrap border-[3px] px-3 py-1 text-lg font-bold uppercase tracking-widest sm:text-xl",
                        " AmritCMS "
                    }
                    nav {
                        "aria-label": "Navigation",
                        class: "hidden tracking-wide xl:flex",
                        h2 { class: "sr-only", id: "navigation", "Navigation" }
                        menu { class: "text-heading flex space-x-10 tracking-wider",
                            li {
                                a {
                                    href: "/en/blog",
                                    class: "inline-flex whitespace-nowrap py-4 text-base font-semibold",
                                    "Link 1"
                                }
                            }
                            li {
                                a {
                                    href: "/en/blog",
                                    class: "inline-flex whitespace-nowrap py-4 text-base font-semibold",
                                    "Link 2"
                                }
                            }
                        }
                    }
                }
                div { class: "text-heading hidden items-center gap-4 font-semibold xl:flex",
                    a {
                        href: "/en/contact",
                        class: "hover:text-heading flex items-center gap-2",
                        span { "Username" }
                    }
                    svg {
                        "xmlns": "http://www.w3.org/2000/svg",
                        "viewBox": "0 0 64 512",
                        class: "h-4",
                        path {
                            "clip-rule": "evenodd",
                            "fill-rule": "evenodd",
                            "d": "M48 0V16 496v16H16V496 16 0H48z",
                            "fill": "currentColor"
                        }
                    }
                    div {
                        button { onclick, class: "flex items-center justify-center gap-2",
                            div { " Logout" }
                        }
                    }
                }
                aside { class: "flex h-full w-1/2 flex-1 justify-end xl:hidden",
                    h2 { class: "sr-only", id: "mobile-navigation", "Mobile navigation" }
                    div { class: "flex items-center",
                        label {
                            r#for: "ss-mobile-menu",
                            class: "relative z-[9999] cursor-pointer px-3 py-6",
                            input {
                                r#type: "checkbox",
                                class: "peer hidden",
                                id: "ss-mobile-menu"
                            }
                            div { class: "bg-heading before:bg-heading after:bg-heading relative z-[10000] block h-[1px] w-7 bg-transparent content-[''] before:absolute before:top-[-0.35rem] before:z-[10000] before:block before:h-full before:w-full before:transition-all before:duration-200 before:ease-out before:content-[''] after:absolute after:bottom-[-0.35rem] after:right-0 after:block after:h-full after:w-full after:transition-all after:duration-200 after:ease-out after:content-[''] peer-checked:bg-transparent before:peer-checked:top-0 before:peer-checked:w-full before:peer-checked:rotate-45 before:peer-checked:transform after:peer-checked:bottom-0 after:peer-checked:w-full after:peer-checked:-rotate-45 after:peer-checked:transform" }
                            div { class: "bg-heading/50 fixed inset-0 z-[9999] hidden h-full w-full backdrop-blur-sm peer-checked:block",
                                " "
                            }
                            div { class: "peer-checked:shadow-heading fixed right-0 top-0 z-[9999] h-full w-full translate-x-full overflow-y-auto overscroll-y-none transition duration-500 peer-checked:translate-x-0",
                                div { class: "float-right min-h-full w-[85%] bg-white px-6 pt-12 shadow-2xl",
                                    menu { class: "text-heading mb-8 mt-8 flex flex-col space-y-4",
                                        li {
                                            a {
                                                href: "/en/blog",
                                                class: "whitespace-nowrap pb-1 font-semibold",
                                                "Link 1"
                                            }
                                        }
                                        li {
                                            a {
                                                href: "/en/blog",
                                                class: "whitespace-nowrap pb-1 font-semibold",
                                                "Link 2"
                                            }
                                        }
                                        li {
                                            a {
                                                href: "/en/blog",
                                                class: "whitespace-nowrap pb-1 font-semibold",
                                                "Username"
                                            }
                                        }
                                        li {
                                            button { onclick, class: "flex items-center justify-center gap-2",
                                                div { " Logout" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
