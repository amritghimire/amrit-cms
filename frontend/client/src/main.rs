#![allow(non_snake_case)]

use client::App;
use dioxus::prelude::*;
use log::LevelFilter;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    LaunchBuilder::new()
        .with_cfg(dioxus::web::Config::new().hydrate(true))
        .launch(App);
}
