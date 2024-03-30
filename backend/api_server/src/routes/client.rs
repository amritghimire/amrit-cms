use axum::http::{StatusCode, Uri};
use axum::response::Html;
use utils::configuration::Settings;

use client::App;
use sycamore::prelude::*;
use tokio::fs;

pub async fn serve_frontend(uri: Uri) -> (StatusCode, Html<String>) {
    let settings = Settings::new().expect("Failed to read configuration");
    let serve_dir_path = settings.frontend.assets.clone();

    tracing::info!(
        "{}/index.html from {:?}",
        &serve_dir_path,
        std::env::current_dir()
    );

    let index_file = fs::read(format!("{}/index.html", &serve_dir_path)).await;
    if index_file.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("Unable to build index".into()),
        );
    }
    let index_file = index_file.unwrap();
    let index_html = String::from_utf8(index_file);
    if index_html.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("Unable to build index from utf8".into()),
        );
    }
    let index_html = index_html.unwrap();

    let rendered = sycamore::render_to_string(|cx| {
        view! { cx,
            App(Some(uri.to_string()))
        }
    });

    let index_html = index_html.replace("%sycamore.body", &rendered);

    (StatusCode::OK, Html(index_html))
}
