use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::config::get_config;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer =
        BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

    let config = get_config().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", config.app_port);

    let listener = TcpListener::bind(address).expect("Failed to bind to port");
    let connection_pool = PgPool::connect(&config.database.connect_string())
        .await
        .expect("Failed to connect to Postgres");

    run(listener, connection_pool)?.await
}
