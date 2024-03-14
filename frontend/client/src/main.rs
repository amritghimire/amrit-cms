use client::app::app;
use dioxus_web::launch::launch_cfg;
use dioxus_web::Config;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    let config = Config::new().hydrate(true);

    launch_cfg(app, config)
}
