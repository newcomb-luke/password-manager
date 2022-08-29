use diesel::prelude::*;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

use crate::{endpoints::check_user_exists, guards::AuthKey, ApiError, DataDatabase, Vault};

#[get("/update_key")]
pub async fn set_new_key(
    conn: DataDatabase,
    old_auth_key: AuthKey,
    new_auth_key: NewAuthKey,
    vault: Vault
) -> Result<String, ApiError> {
    conn.run(move |c| update_key_in_db(c, old_auth_key, new_auth_key, vault))
        .await
}

fn update_key_in_db(
    conn: &diesel::SqliteConnection,
    old_auth_key: AuthKey,
    new_auth_key: NewAuthKey,
    new_vault: Vault
) -> Result<String, ApiError> {
    use crate::schema::users::dsl::*;

    trace!("Updating user authentication key in database");

    if check_user_exists(conn, old_auth_key)? {
        diesel::update(users.filter(key.eq(String::from(old_auth_key))))
            .set((key.eq(String::from(new_auth_key)), vault.eq(new_vault.0)))
            .execute(conn)
            .map_err(|_| {
                error!("Error updating authentication key for user in database");

                ApiError::DatabaseWrite
            })
            .map(|_| {
                info!("Updated authentication key in database");

                String::from("Success")
            })
    } else {
        error!(
            "Attempted to update authentication key of user with authentication key not in database"
        );

        Err(ApiError::UserNoExists)
    }
}

/// This represents a new authentication key that will be set by the user's request
#[derive(Clone, Copy)]
pub struct NewAuthKey(pub [u8; 32]);

impl From<NewAuthKey> for String {
    fn from(key: NewAuthKey) -> Self {
        hex::encode(key.0)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for NewAuthKey {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("x-new-auth-key") {
            Some(key_str) => {
                let mut key_bytes = [0; 32];

                if hex::decode_to_slice(key_str, &mut key_bytes).is_err() {
                    Outcome::Failure((Status::BadRequest, ApiError::AuthKeyInvalid))
                } else {
                    Outcome::Success(NewAuthKey(key_bytes))
                }
            }
            None => Outcome::Failure((Status::BadRequest, ApiError::AuthKeyMissing)),
        }
    }
}
