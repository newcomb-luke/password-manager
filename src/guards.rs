use crate::ApiError;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

/// This represents an authentication key that will be used to request and update the user's vault
#[derive(Clone, Copy)]
pub struct AuthKey(pub [u8; 32]);

impl From<AuthKey> for String {
    fn from(key: AuthKey) -> Self {
        hex::encode(key.0)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthKey {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("x-auth-key") {
            Some(key_str) => {
                let mut key_bytes = [0; 32];

                if hex::decode_to_slice(key_str, &mut key_bytes).is_err() {
                    Outcome::Failure((Status::BadRequest, ApiError::AuthKeyInvalid))
                } else {
                    Outcome::Success(AuthKey(key_bytes))
                }
            }
            None => Outcome::Failure((Status::BadRequest, ApiError::AuthKeyMissing)),
        }
    }
}

/// This is a wrapper struct for our Email type so that it can be turned into a request guard
#[derive(Clone)]
pub struct Email(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Email {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("x-email") {
            Some(email_str) => {
                let halves: Vec<_> = email_str.split('@').collect();

                // Every e-mail has to have ___@___.com, so here we validate that it has both
                // halves
                if halves.len() != 2 {
                    Outcome::Failure((Status::BadRequest, ApiError::EmailInvalid))
                } else {
                    // Checks for the __.__ at the end of the e-mail
                    let mut halves_iter = halves.iter();
                    // We have already checked that the length is most definitely 2, so we can
                    // ignore the first part and call .unwrap()
                    halves_iter.next().unwrap();

                    let url_half = halves_iter.next().unwrap();

                    if url_half.split('.').count() != 2 {
                        Outcome::Failure((Status::BadRequest, ApiError::EmailInvalid))
                    } else {
                        Outcome::Success(Email(String::from(email_str)))
                    }
                }
            }
            None => Outcome::Failure((Status::BadRequest, ApiError::EmailMissing)),
        }
    }
}

/// This is a wrapper struct for our Vault type so that it can be turned into a request guard
#[derive(Clone)]
pub struct Vault(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Vault {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("x-vault") {
            Some(vault_str) => {
                // We just check if it is a valid hexadecimal string
                if hex::decode(vault_str).is_ok() {
                    Outcome::Success(Vault(String::from(vault_str)))
                } else {
                    Outcome::Failure((Status::BadRequest, ApiError::VaultInvalid))
                }
            }
            None => Outcome::Failure((Status::BadRequest, ApiError::VaultMissing)),
        }
    }
}
