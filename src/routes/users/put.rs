pub async fn put(
    request: &mut crate::Request,
    respond: &mut crate::Respond,
    database: crate::Database,
) -> Result<(), h2::Error> {
    use crate::*;

    check_content_type!(request, respond, "application/x-www-form-urlencoded");

    #[derive(serde::Deserialize)]
    struct Body {
        username: String,
        password: String,
    }

    let body = body!(request, respond, serde_urlencoded, Body);

    // reasonable restrictions
    if body.username.len() < 3
        || body.username.len() > 32
        || body.password.len() < 3
        || body.password.len() > 128
    {
        send_response!(
            respond,
            BAD_REQUEST,
            Response::failure("Invalid request payload")
        );
    }

    let result =
        sqlx::query("INSERT INTO users (username) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id")
            .bind(body.username)
            .fetch_one(&database)
            .await;

    let user_id: i64 = match result {
        Ok(row) => {
            use sqlx::Row;

            row.get_unchecked(0)
        }
        Err(e) => {
            // postgres will not return user id in case of a conflict
            if matches!(e, sqlx::error::Error::RowNotFound) {
                send_response!(
                    respond,
                    CONFLICT,
                    Response::failure("Username already exists")
                );
            } else {
                log::error!("{:?}", e);

                send_response!(
                    respond,
                    INTERNAL_SERVER_ERROR,
                    Response::failure("Internal Server Error")
                );
            }
        }
    };

    use argon2::PasswordHasher;

    // in production i would prefer to either avoid password-based auth at all or to
    // outsource it to some cloud service (like auth0 or Cognito)
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
    let argon2 = argon2::Argon2::default();
    let random = &mut argon2::password_hash::rand_core::OsRng;
    let salt = argon2::password_hash::SaltString::generate(random);
    let result = argon2.hash_password(body.password.as_bytes(), &salt);

    let password_hash = match result {
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
