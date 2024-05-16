use std::net::{AddrParseError, SocketAddr};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Database connection URL.
    #[clap(env)]
    pub database_url: String,
    /// HTTP server address.
    #[clap(env, short, long, default_value = "0.0.0.0:8080")]
    #[arg(value_parser = parse_addr)]
    pub server_address: SocketAddr,
    /// Maximum number of database connections.
    #[clap(env, short, long, default_value_t = 10)]
    pub max_connections: u32,
    /// Generate the database schema and exit.
    #[clap(long, default_value_t = false)]
    pub generate_schema: bool,
    /// Postgres sslmode setting.
    #[clap(env, long, default_value = "prefer")]
    pub sslmode: String,
}

fn parse_addr(arg: &str) -> Result<SocketAddr, AddrParseError> {
    arg.parse()
}
