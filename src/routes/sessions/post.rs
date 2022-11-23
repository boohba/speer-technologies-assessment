use crate::common::*;
use argon2::{PasswordHash, PasswordVerifier};
use sqlx::Row;

pub async fn post(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    check_content_type!(request, respond);

    let credentials = body!(request, respond, Credentials);

    if credentials.is_invalid() {
        send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
    }

    let result = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(credentials.username)
        .fetch_optional(&database)
        .await;

    let row = match unwrap_internal_error!(respond, result) {
        Some(row) => row,
        None => {
            send_response!(respond, NOT_FOUND, Response::NOT_FOUND);
        }
    };

    let hash = row.get_unchecked::<String, _>(1);
    let hash = unwrap_internal_error!(respond, PasswordHash::new(&hash));

    if let Err(_) = ARGON2.verify_password(credentials.password.as_bytes(), &hash) {
        send_response!(respond, UNAUTHORIZED, Response::UNAUTHORIZED);
    }

    let result = sqlx::query("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
        .bind(row.get_unchecked::<i64, _>(0))
        .fetch_one(&database)
        .await;

    let session_id = unwrap_internal_error!(respond, result)
        .get_unchecked::<i64, _>(0)
        .to_le_bytes();

    let token = auth::create_token(&session_id);

    send_response!(respond, CREATED, Response::success(token));
}
