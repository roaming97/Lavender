use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderName, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sha3::{Digest, Sha3_256};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, PartialEq, Deserialize)]
pub struct ApiKey(pub String);

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Empty,
    Invalid,
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        let body = match self {
            ApiKeyError::Invalid => (StatusCode::BAD_REQUEST, "Invalid Lavender API Key!"),
            ApiKeyError::Empty => (
                StatusCode::UNAUTHORIZED,
                "Provided Lavender API Key is empty!",
            ),
            ApiKeyError::Missing => (StatusCode::UNAUTHORIZED, "Lavender API Key is missing!"),
        };

        body.into_response()
    }
}

impl ApiKey {
    /// Checks if the API key is valid by testing it against a hash.
    fn validate(&self, hash: &str) -> Result<(), ApiKeyError> {
        let k = &self.0;
        if k.is_empty() {
            return Err(ApiKeyError::Empty);
        }

        if format!("{:x}", Sha3_256::digest(k.as_bytes())) == hash {
            Ok(())
        } else {
            Err(ApiKeyError::Invalid)
        }
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for ApiKey {
    type Rejection = (StatusCode, ApiKeyError);

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let header_name = HeaderName::from_static("lav-api-key");
        if let Some(value) = parts.headers.get(&header_name) {
            let api_key = value.to_str().map_err(|_| ApiKeyError::Invalid).unwrap();
            if api_key.is_empty() {
                return Err((StatusCode::UNAUTHORIZED, ApiKeyError::Empty));
            }
            let api_key = ApiKey(api_key.to_owned());
            let hash = &state.lavender_api_hash;
            match api_key.validate(hash) {
                Ok(_) => Ok(api_key),
                Err(e) => Err((StatusCode::BAD_REQUEST, e)),
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, ApiKeyError::Missing))
        }
    }
}
