mod api;
mod file;
mod routes;
use std::sync::Arc;
#[cfg(test)]
mod tests;

use axum::{routing::get, Router};
use routes::{file_amount, get_file, get_latest_files};
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::SecretStore;

pub struct ShuttleState {
    pub config: Config,
    pub secrets: SecretStore
}

impl ShuttleState {
    pub fn new(config: Config, secrets: SecretStore) -> Self {
        Self {
            config,
            secrets
        }
    }
}

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
fn lavender(state: Arc<ShuttleState>) -> Router {
    Router::new()
        .route("/amount", get(file_amount))
        .route("/file", get(get_file))
        .route("/latest", get(get_latest_files))
        .with_state(state)
}

#[shuttle_runtime::main]
pub async fn axum(#[shuttle_runtime::Secrets] secrets: SecretStore) -> ShuttleAxum {
    let config = Config::new();
    let state = Arc::<ShuttleState>::new(ShuttleState::new(config, secrets));

    let lavender = lavender(state);

    Ok(lavender.into())
}
