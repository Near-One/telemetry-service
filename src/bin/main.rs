use clap::Parser;
use telemetry_service::{
    database::{connect_and_refresh_schema, MAINNET_DB_NAME, TESTNET_DB_NAME},
    Config, Error, Server,
};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();

    let config = Config::parse();

    let db_mainnet = connect_and_refresh_schema(
        &config.database_url,
        MAINNET_DB_NAME,
        config.max_connections,
    )
    .await?;
    let db_testnet = connect_and_refresh_schema(
        &config.database_url,
        TESTNET_DB_NAME,
        config.max_connections,
    )
    .await?;

    if config.generate_schema {
        info!("generated database schema - now exiting");
        return Ok(());
    }

    let http_server = Server::new(config.server_address, db_mainnet, db_testnet)?;
    http_server.run().await
}

fn setup_tracing() {
    let fmt_layer = fmt::layer().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,sqlx=warn"))
        .expect("failed to create env filter for tracing");
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
