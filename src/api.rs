use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use sha3::{Digest, Sha3_256};
use std::env;

#[derive(Debug, PartialEq)]
pub struct ApiKey(String);

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    MissingEnv,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let hash = match env::var("LAVENDER_API_HASH") {
            Ok(s) => s,
            Err(_) => {
                println!("Lavender API hash is missing from environment variables!");
                return Outcome::Failure((Status::InternalServerError, ApiKeyError::MissingEnv));
            }
        };

        match req.headers().get_one("lv-api-key") {
            Some(k) if format!("{:x}", Sha3_256::digest(k.as_bytes())) == hash => {
                Outcome::Success(ApiKey(k.to_owned()))
            }
            Some(_) => Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid)),
            None => Outcome::Failure((Status::Unauthorized, ApiKeyError::Missing)),
        }
    }
}
