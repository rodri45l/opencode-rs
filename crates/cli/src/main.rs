//! `opencode-rs` — CLI entry point.

use anyhow::Context;
use clap::{Parser, Subcommand};
use opencode_server::{router, AppState};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(
    name = "opencode-rs",
    version,
    about = "A Rust implementation of the opencode server"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the HTTP API server.
    Serve {
        /// Host to bind.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind.
        #[arg(long, default_value_t = 8081)]
        port: u16,
    },
    /// Check that a running server is healthy.
    Health {
        /// Base URL of a running server.
        #[arg(long, default_value = "http://127.0.0.1:8081")]
        base_url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match Cli::parse().command {
        Command::Serve { host, port } => {
            let addr: SocketAddr = format!("{host}:{port}")
                .parse()
                .context("invalid bind address")?;
            let listener = tokio::net::TcpListener::bind(addr).await?;
            tracing::info!("opencode-rs listening on http://{addr}");
            axum::serve(listener, router(AppState::new())).await?;
        }
        Command::Health { base_url } => {
            let client = opencode_client::Client::new(base_url);
            let health = client.health().await?;
            println!("healthy={}", health.healthy);
        }
    }

    Ok(())
}
