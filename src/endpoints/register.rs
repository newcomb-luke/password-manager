use diesel::prelude::*;

use crate::{
    models::{NewUser, User},
    ApiError, AuthKey, DataDatabase, Email, Vault,
};

#[get("/register")]
pub async fn register_user(
    conn: DataDatabase,
    email: Email,
    auth_key: AuthKey,
    vault: Vault,
) -> Result<String, ApiError> {
    conn.run(move |c| register_new_user(c, email, auth_key, vault))
        .await
}

fn register_new_user(
    conn: &diesel::SqliteConnection,
    email: Email,
    auth_key: AuthKey,
    vault: Vault,
) -> Result<String, ApiError> {
    use crate::schema::users;

    trace!("Registering new user");

    // Check to see if the user already exists in the database before inserting
    if check_email_exists(conn, &email)? {
        error!("User already exists in database");

        Err(ApiError::UserExists)
    } else {
        let key: String = auth_key.into();

        let new_user = NewUser {
            email: email.0,
            key,
            vault: vault.0,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .map_err(|_| {
                error!("Failed to write new user to database");

                ApiError::DatabaseWrite
            })
            .map(|_| {
                info!("Successfully registered new user in database");

                String::from("Success")
            })
    }
}

pub fn check_email_exists(
    conn: &diesel::SqliteConnection,
    check_email: &Email,
) -> Result<bool, ApiError> {
    use crate::schema::users::dsl::*;

    trace!("Checking if email exists in database");

    let user_list = users
        .filter(email.eq(&check_email.0))
        .load::<User>(conn)
        .map_err(|_| {
            error!("Failed to read database");

            ApiError::DatabaseRead
        })?;

    if user_list.len() > 1 {
        error!("INTERNAL ERROR: Two users cannot have identical emails");

        Err(ApiError::InternalError)
    } else {
        Ok(user_list.len() == 1)
    }
}
