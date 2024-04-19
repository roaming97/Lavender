use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderName, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sha3::{Digest, Sha3_256};
use std::sync::Arc;

use crate::ShuttleState;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Key(pub String);

#[derive(Debug)]
pub enum KeyError {
    Missing,
    Empty,
    Invalid,
    MissingEnv
}

impl IntoResponse for KeyError {
    fn into_response(self) -> Response {
        let body = match self {
            Self::Invalid => (StatusCode::BAD_REQUEST, "Invalid Lavender API Key!"),
            Self::Empty => (
                StatusCode::UNAUTHORIZED,
                "Provided Lavender API Key is empty!",
            ),
            Self::Missing => (StatusCode::UNAUTHORIZED, "Lavender API Key is missing!"),
            Self::MissingEnv => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lavender API Key is missing from the environment variables!",
            ),
        };

        body.into_response()
    }
}

impl Key {
    /// Checks if the API key is valid by testing it against a hash.
    fn validate(&self, hash: &str) -> Result<(), KeyError> {
        let k = &self.0;
        if k.is_empty() {
            return Err(KeyError::Empty);
        }

        if format!("{:x}", Sha3_256::digest(k.as_bytes())) == hash {
            Ok(())
        } else {
            Err(KeyError::Invalid)
        }
    }
}

#[async_trait]
impl FromRequestParts<Arc<ShuttleState>> for Key {
    type Rejection = (StatusCode, KeyError);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ShuttleState>,
    ) -> Result<Self, Self::Rejection> {
        let header_name = HeaderName::from_static("lav-api-key");
        if let Some(value) = parts.headers.get(&header_name) {
            let api_key = value.to_str().map_err(|_| KeyError::Invalid).unwrap();
            if api_key.is_empty() {
                return Err((StatusCode::UNAUTHORIZED, KeyError::Empty));
            }
            let api_key = Self(api_key.to_owned());

            if let Some(hash) = state.secrets.get("LAVENDER_API_HASH") {
                match api_key.validate(&hash) {
                    Ok(()) => Ok(api_key),
                    Err(e) => Err((StatusCode::BAD_REQUEST, e)),
                }
            } else {
                eprintln!("Lavender API key is missing from Secrets.toml!");
                Err((StatusCode::BAD_REQUEST, KeyError::MissingEnv))
            }

        } else {
            Err((StatusCode::UNAUTHORIZED, KeyError::Missing))
        }
    }
}
