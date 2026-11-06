mod api;
mod configuration;
mod db;

use std::time::Duration;

use anyhow::Context;
use configuration::Config;
use mongodb::{
    Client,
    bson::self,
    options::ClientOptions,
};
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt};

const SHUTDOWN_DELAY: Duration = Duration::from_secs(5);

async fn shutdown_signal() {
    let ctrl_c_signal = async {
        ctrl_c()
            .await
            .expect("Failed to install CTRL+c handler for graceful shutdown.");
    };

    #[cfg(unix)]
    let terminate = async {
        signal(SignalKind::terminate())
            .expect("Failed to install signal for graceful shutdown")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! { _ = ctrl_c_signal => {}, _ = terminate => {} }
    tracing::info!("Shutdown signal received. Shutting down in 3 secs..");

    // Including here a sleep to make sure probes are completed
    tokio::time::sleep(SHUTDOWN_DELAY).await;

    tracing::info!("Shutdown completed. Bye.")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    // Log init
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("LOG_LEVEL")
        .from_env_lossy();
    fmt().with_env_filter(env_filter).init();

    // Configuration init
    let configuration_file_path =
        std::env::var("CONFIGURATION_FILE").unwrap_or_else(|_| "./config.json".into());

    let config =
        Config::parse_config(configuration_file_path).context("Reading configuration file")?;

    // MongoDb init
    let mongodb_connection_string = std::env::var("MONGODB_CONNECTION_STRING").context(
        "Environment Variable MONGODB_CONNECTION_STRING not found. Impossible to initialize MongoDB."
    )?;

    let mut options = ClientOptions::parse(mongodb_connection_string).await?;

    if let Some(app_name) = config.api.object.app_name {
        options.app_name = Some(app_name);
    };

    let client = Client::with_options(options)?;
    let database = client.database(&config.api.object.database);

    let collection = database.collection::<bson::Document>(&config.api.object.collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    // TODO: Add metrics APIs

    axum::serve(
        listener,
        api::object::router(collection).merge(api::health::router(database)),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}
