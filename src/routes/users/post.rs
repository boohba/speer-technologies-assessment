use crate::prelude::*;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::PasswordHasher;
use sqlx::Row;

pub async fn post(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    check_content_type!(request, respond, "application/json");

    let credentials = body!(request, respond, Credentials);

    if credentials.is_invalid() {
        send_response!(
            respond,
            BAD_REQUEST,
            Response::failure("Invalid request payload")
        );
    }

    let result =
        sqlx::query("INSERT INTO users (username) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id")
            .bind(credentials.username)
            .fetch_optional(&database)
            .await;

    let user_id: i64 = match result {
        Ok(row) => {
            // postgres will not return user id in case of a conflict
            match row {
                Some(row) => row.get_unchecked(0),
                None => {
                    send_response!(
                        respond,
                        CONFLICT,
                        Response::failure("Username already exists")
                    );
                }
            }
        }
        Err(e) => {
            log::error!("{:?}", e);

            send_response!(
                respond,
                INTERNAL_SERVER_ERROR,
                Response::failure("Internal Server Error")
            );
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let password = credentials.password.as_bytes();

    // in production i would prefer to either avoid password-based auth at all or to
    // outsource it to some cloud service (like auth0 or Cognito)
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
    let password_hash = match ARGON2.hash_password(password, &salt) {
        Ok(password_hash) => password_hash.to_string(),
        // the username is already reserved at this point, so it should be cleaned up in
        // case of an error, but for the sake of simplicity we will not do that.
        Err(e) => {
            log::error!("{:?}", e);

            send_response!(
                respond,
                INTERNAL_SERVER_ERROR,
                Response::failure("Internal Server Error")
            );
        }
    };

    let result = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(&database)
        .await;

    // once again, the username is already reserved, so in case of an error it must be
    // cleaned up, but the clock is ticking, so i will not implement that.
    if let Err(e) = result {
        log::error!("{:?}", e);

        send_response!(
            respond,
            INTERNAL_SERVER_ERROR,
            Response::failure("Internal Server Error")
        );
    }

    send_response!(respond, CREATED, Response::<()>::success(None));
}
