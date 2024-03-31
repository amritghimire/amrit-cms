#![allow(non_snake_case)]

use phf::phf_map;
use sycamore::prelude::*;

const CLOSED_MENU: &str = include_str!("closed_menu.svg");
const OPEN_MENU: &str = include_str!("open_menu.svg");

static SVG_MAP: phf::Map<&'static str, &'static str> = phf_map! {
  "closed_menu" => CLOSED_MENU, "open_menu" => OPEN_MENU
};

#[component(inline_props)]
pub fn SvgIcon<G: Html>(icon: String, attributes: Attributes<G>) -> View<G> {
    let icon_svg = SVG_MAP.get(&icon).unwrap_or(&"");
    view! {
        span(..attributes, dangerously_set_inner_html=*icon_svg) {}
    }
}
