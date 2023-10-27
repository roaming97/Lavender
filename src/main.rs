mod api;
mod file;
mod routes;
use std::sync::Arc;
#[cfg(test)]
mod tests;

use axum::{routing::get, Router};
use file::LavenderConfig;
use routes::*;

pub struct AppState {
    config: LavenderConfig,
    lavender_api_hash: String
}

/// A lavender blooms from the rusty soil.
fn lavender(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/amount", get(file_amount))
        .route("/file", get(get_file))
        .route("/latest", get(get_latest_files))
        .route("/optimize", get(create_optimized_images))
        .with_state(state)
}

#[shuttle_runtime::main]
pub async fn axum(
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let config = LavenderConfig::new();
    let lavender_api_hash = secrets.get("LAVENDER_API_HASH").expect("Environment variable LAVENDER_API_HASH");
    let state = Arc::<AppState>::new(AppState { config, lavender_api_hash });

    Ok(lavender(state).into())
}