mod api;
mod file;
mod routes;
use std::sync::Arc;
#[cfg(test)]
mod tests;

use axum::{routing::get, Router};
use color_eyre::eyre::Result;
use routes::{file_amount, get_file, get_latest_files};
use tokio::signal;

/// Server configuration structure, deserializes `lavender.toml` into it.
#[derive(serde::Deserialize, Default)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub media_path: String,
}

impl Config {
    #[must_use]
    /// Creates a new Lavender configuration from a `lavender.toml` file.
    /// # Panics
    ///
    /// Will panic if there is no `lavender.toml` found at the root directory of the project.
    pub fn new() -> Self {
        let toml_str =
            std::fs::read_to_string("lavender.toml").expect("Failed to read configuration TOML");
        let config: Self =
            toml::from_str(&toml_str).expect("Failed to deserialize configuration TOML.");
        config
    }
}

/// A lavender blooms from the soil.
fn lavender(state: Arc<Config>) -> Router {
    Router::new()
        .route("/amount", get(file_amount))
        .route("/file", get(get_file))
        .route("/latest", get(get_latest_files))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new();
    let address = config.address.clone();
    let port = config.port;
    let state = Arc::<Config>::new(config);

    let lavender = lavender(state);

    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    axum::serve(listener, lavender)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("Detected Ctrl+C, shutting down gracefully");
}
