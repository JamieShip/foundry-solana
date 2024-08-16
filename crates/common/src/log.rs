use tracing::{level_filters, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[allow(dead_code)]
pub fn subscriber() {
    tracing_subscriber::registry()
        .with(level_filters::LevelFilter::from_level(Level::INFO))
        .with(fmt::layer())
        .init();
}
