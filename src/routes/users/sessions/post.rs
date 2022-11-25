use crate::common::*;
use argon2::{PasswordHash, PasswordVerifier};

pub async fn post(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let credentials = body!(request, Credentials);

    if credentials.is_invalid() {
        return Ok(Response::bad_request());
    }

    let result = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(credentials.username)
        .fetch_optional(&database)
        .await;

    let row = match unwrap_internal_error!(result) {
        Some(row) => row,
        None => {
            return Ok(Response::not_found());
        }
    };

    let hash = row.get_unchecked::<String, _>(1);
    let hash = unwrap_internal_error!(PasswordHash::new(&hash));

    if let Err(_) = ARGON2.verify_password(credentials.password.as_bytes(), &hash) {
        return Ok(Response::not_found());
    }

    let result = sqlx::query("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
        .bind(row.get_unchecked::<i64, _>(0))
        .fetch_one(&database)
        .await;

    let session_id = unwrap_internal_error!(result)
        .get_unchecked::<i64, _>(0)
        .to_le_bytes();

    let token = auth::create_token(&session_id);

    Ok((StatusCode::CREATED, Response::success(token)))
}
