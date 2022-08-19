use diesel::prelude::*;

use crate::{guards::AuthKey, models::User, ApiError, DataDatabase};

#[get("/auth")]
pub async fn authenticate(conn: DataDatabase, auth_key: AuthKey) -> Result<&'static str, ApiError> {
    conn.run(move |c| authenticate_key(c, auth_key)).await
}

fn authenticate_key(
    conn: &diesel::SqliteConnection,
    auth_key: AuthKey,
) -> Result<&'static str, ApiError> {
    Ok(if check_user_exists(conn, auth_key)? {
        "1"
    } else {
        "0"
    })
}

pub fn check_user_exists(
    conn: &diesel::SqliteConnection,
    auth_key: AuthKey,
) -> Result<bool, ApiError> {
    use crate::schema::users::dsl::*;

    trace!("Checking if user exists in database");

    let user_list = users
        .filter(key.eq(String::from(auth_key)))
        .load::<User>(conn)
        .map_err(|_| {
            error!("Failed to read database");

            ApiError::DatabaseRead
        })?;

    if user_list.len() > 1 {
        error!("INTERNAL ERROR: Two users cannot have identical authentication keys");

        Err(ApiError::InternalError)
    } else {
        Ok(user_list.len() == 1)
    }
}
