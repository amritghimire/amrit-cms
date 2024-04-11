#![allow(non_snake_case)]

mod app;
pub(crate) mod components;
pub(crate) mod entities;
pub(crate) mod errors;
pub(crate) mod pages;
pub(crate) mod routes;
pub(crate) mod state;
pub(crate) mod utils;

pub use app::App;

pub type Result<T> = std::result::Result<T, errors::ApplicationError>;
