use diesel::prelude::*;

use crate::{guards::AuthKey, models::User, ApiError, DataDatabase};

#[get("/get_vault")]
pub async fn request_vault(conn: DataDatabase, auth_key: AuthKey) -> Result<String, ApiError> {
    conn.run(move |c| read_vault_from_db(c, auth_key)).await
}

fn read_vault_from_db(
    conn: &diesel::SqliteConnection,
    auth_key: AuthKey,
) -> Result<String, ApiError> {
    use crate::schema::users::dsl::*;

    trace!("Reading vault from database");

    let mut user_list = users
        .filter(key.eq(String::from(auth_key)))
        .load::<User>(conn)
        .map_err(|_| {
            error!("Failed to read database");

            ApiError::DatabaseRead
        })?;

    if user_list.len() > 1 {
        error!("INTERNAL ERROR: Multiple users should not have identical authorization key");

        Err(ApiError::InternalError)
    } else {
        let user = user_list.pop().ok_or_else(|| {
            error!("Attempted to request vault from user with authentication key not in database");

            ApiError::UserNoExists
        })?;

        Ok(user.vault)
    }
}
