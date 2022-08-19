use diesel::prelude::*;

use crate::{
    endpoints::check_user_exists,
    guards::{AuthKey, Vault},
    ApiError, DataDatabase,
};

#[get("/update_vault")]
pub async fn set_new_vault(
    conn: DataDatabase,
    auth_key: AuthKey,
    new_vault: Vault,
) -> Result<String, ApiError> {
    conn.run(move |c| update_vault_in_db(c, auth_key, new_vault))
        .await
}

fn update_vault_in_db(
    conn: &diesel::SqliteConnection,
    auth_key: AuthKey,
    new_vault: Vault,
) -> Result<String, ApiError> {
    use crate::schema::users::dsl::*;

    trace!("Updating vault in database");

    if check_user_exists(conn, auth_key)? {
        diesel::update(users.filter(key.eq(String::from(auth_key))))
            .set(vault.eq(new_vault.0))
            .execute(conn)
            .map_err(|_| {
                error!("Error updating vault for user in database");

                ApiError::DatabaseWrite
            })
            .map(|_| {
                info!("Updated vault in database");

                String::from("Success")
            })
    } else {
        error!("Attempted to update vault of user with authentication key not in database");

        Err(ApiError::UserNoExists)
    }
}
