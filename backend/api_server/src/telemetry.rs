use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::Registry;

pub fn init_subscriber(subscriber: Layered<Layer<Registry>, Registry>) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn get_subscriber() -> Layered<Layer<Registry>, Registry> {
    Registry::default().with(tracing_subscriber::fmt::layer())
}
