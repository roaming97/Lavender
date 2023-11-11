mod api;
mod file;
mod routes;
use std::sync::Arc;
#[cfg(test)]
mod tests;

use axum::{routing::get, Router};
use file::LavenderConfig;
use routes::*;
use tokio::signal;

/// A lavender blooms from the rusty soil.
fn lavender(state: Arc<LavenderConfig>) -> Router {
    Router::new()
        .route("/amount", get(file_amount))
        .route("/file", get(get_file))
        .route("/latest", get(get_latest_files))
        .route("/optimize", get(create_optimized_images))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let config = LavenderConfig::new();
    let address = config.server.address.to_owned();
    let port = config.server.port;
    let state = Arc::<LavenderConfig>::new(config);

    let lavender = lavender(state);

    let addr = &format!("{address}:{}", port).parse().unwrap();
    axum::Server::bind(addr)
        .serve(lavender.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Detected Ctrl+C, shutting down gracefully");
}
