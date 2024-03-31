#![allow(non_snake_case)]

use crate::nav::MenuButton;
use sycamore::prelude::*;

#[component]
pub fn Navbar<G: Html>() -> View<G> {
    view! {
        nav (class="bg-gray-800") {
            div (class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8") {
                MenuButton() {}
                div (class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start") {
                    div (class="flex flex-shrink-0 items-center") {
                        "amrit-cms"
                    }
                    div (class="hidden sm:ml-6 sm:block") {
                        div (class="flex space-x-4") {
                            a (class="bg-gray-900 text-white rounded-md px-3 py-2 text-sm font-medium", href="#", aria-current="page") {
                            "Home"
                            }
                        }
                    }
                }
            }
            div (class="sm:hidden", id="mobile-menu") {
                div (class="space-y-1 px-2 pb-3 pt-2") {
                    a (class="bg-gray-900 text-white block rounded-md px-3 py-2 text-base font-medium", href="#", aria-current="page") {
                        "Home"
                    }
                }
            }
        }
    }
}
