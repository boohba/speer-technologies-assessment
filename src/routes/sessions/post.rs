use crate::prelude::*;
use argon2::{PasswordHash, PasswordVerifier};
use hmac::{Hmac, Mac};
use sha2::Sha256;
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

    let result = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(credentials.username)
        .fetch_optional(&database)
        .await;

    let row = match result {
        Ok(row) => match row {
            Some(row) => row,
            None => {
                send_response!(respond, NOT_FOUND, Response::failure("Not Found"));
            }
        },
        Err(e) => {
            log::error!("{:?}", e);

            send_response!(
                respond,
                INTERNAL_SERVER_ERROR,
                Response::failure("Internal Server Error")
            );
        }
    };

    let user_id = row.get_unchecked::<i64, _>(0);
    let password_hash = row.get_unchecked::<String, _>(1);

    let password_hash = match PasswordHash::new(&password_hash) {
        Ok(password_hash) => password_hash,
        Err(e) => {
            log::error!("{:?}", e);

            send_response!(
                respond,
                INTERNAL_SERVER_ERROR,
                Response::failure("Internal Server Error")
            );
        }
    };

    if let Err(_) = ARGON2.verify_password(credentials.password.as_bytes(), &password_hash) {
        send_response!(respond, UNAUTHORIZED, Response::failure("Unauthorized"));
    }

    let result = sqlx::query("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
        .bind(user_id)
        .fetch_one(&database)
        .await;

    let session_id = match result {
        Ok(row) => row.get_unchecked::<i64, _>(0).to_le_bytes(),
        Err(e) => {
            log::error!("{:?}", e);

            send_response!(
                respond,
                INTERNAL_SERVER_ERROR,
                Response::failure("Internal Server Error")
            );
        }
    };

    let mut mac = Hmac::<Sha256>::new_from_slice(AUTH_SECRET.as_bytes()).unwrap();
    mac.update(&session_id);

    let result = mac.finalize().into_bytes();

    // session_id + hmacsha256 signature, kind of like JWT but more efficient
    let mut auth_token = [0u8; 72];

    unsafe {
        std::ptr::copy_nonoverlapping(session_id.as_ptr(), auth_token.as_mut_ptr(), 8);
        std::ptr::copy_nonoverlapping(result.as_ptr(), auth_token.as_mut_ptr(), 64);
    }

    send_response!(
        respond,
        CREATED,
        Response::success(Some(base64::encode(auth_token)))
    );
}
