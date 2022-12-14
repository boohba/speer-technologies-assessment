use crate::common::*;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::PasswordHasher;

pub async fn post(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let credentials = body!(request, Credentials);

    if credentials.is_invalid() {
        return Ok(Response::bad_request());
    }

    // if an in-progress transaction goes out of scope, it will rollback automatically
    let transaction = unwrap_internal_error!(database.begin().await);

    let result =
        sqlx::query("INSERT INTO users (username) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id")
            .bind(&credentials.username)
            .fetch_optional(&database)
            .await;

    let user_id = match unwrap_internal_error!(result) {
        Some(row) => row.get_unchecked::<i64, _>(0),
        None => {
            return Ok((
                StatusCode::CONFLICT,
                Response::error("Username already exists"),
            ));
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let password = credentials.password.as_bytes();

    // in production i would prefer to either avoid password-based auth at all or to
    // outsource it to some cloud service (like auth0 or Cognito)
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
    let hash = unwrap_internal_error!(ARGON2.hash_password(password, &salt)).to_string();

    let result = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hash)
        .bind(user_id)
        .execute(&database)
        .await;

    unwrap_internal_error!(result);

    tokio::spawn(transaction.commit());

    let response = json!({ "id": user_id, "username": credentials.username });

    Ok((StatusCode::CREATED, Response::success(response)))
}
