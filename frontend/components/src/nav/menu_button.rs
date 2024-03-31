#![allow(non_snake_case)]

use crate::svg::SvgIcon;
use sycamore::prelude::*;

#[component]
pub fn MenuButton<G: Html>() -> View<G> {
    view! {
        div (class="absolute inset-y-0 left-0 flex items-center sm:hidden") {
            button (class="relative inline-flex items-center justify-centerrounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white", aria-controls="mobile-menu", aria-expanded="false", type="button") {
                span (class="absolute -inset-0.5" ) {
                }
                span (class="sr-only" ) {
                    "Open main menu"
                }
                SvgIcon(icon="closed_menu".to_string(), attr:class="block h-6 w-6") {}
                SvgIcon(icon="open_menu".to_string(), attr:class="hidden h-6 w-6") {}
            }
        }
    }
}
