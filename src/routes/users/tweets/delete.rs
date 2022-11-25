use crate::common::*;

pub async fn delete(request: &mut Request, database: Database) -> Result {
    let id = parse_path_var!(request, i64);
    let session_id = check_auth_token!(request);

    let result = sqlx::query("DELETE FROM tweets WHERE id = $1 AND user_id = (SELECT user_id FROM sessions WHERE id = $2)")
        .bind(id)
        .bind(session_id)
        .execute(&database)
        .await;

    if unwrap_internal_error!(result).rows_affected() == 0 {
        return Ok(Response::not_found());
    }

    Ok((StatusCode::OK, Response::empty()))
}
