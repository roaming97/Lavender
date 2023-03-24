use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use sha3::{Digest, Sha3_256};
use std::env;

#[derive(Debug, PartialEq, FromForm)]
pub struct ApiKey<'r>(&'r str);

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    MissingEnv,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let hash = match env::var("LAVENDER_API_HASH") {
            Ok(s) => s,
            Err(_) => {
                println!("Lavender API hash is missing from environment variables!");
                return Outcome::Failure((Status::InternalServerError, ApiKeyError::MissingEnv));
            }
        };

        fn is_valid(key: &str, hash: &str) -> bool {
            format!("{:x}", Sha3_256::digest(key.as_bytes())) == hash
        }

        match req.headers().get_one("x-api-key") {
            Some(k) => {
                if is_valid(k, &hash) {
                    Outcome::Success(ApiKey(k))
                } else {
                    Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid))
                }
            }
            None => Outcome::Failure((Status::Unauthorized, ApiKeyError::Missing)),
        }
    }
}
