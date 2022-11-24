use crate::common::*;

pub async fn patch(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let id = parse_path_var!(request, i64);
    let session_id = check_auth_token!(request);

    #[derive(serde::Deserialize)]
    pub struct Body {
        text: String,
    }

    let body = body!(request, Body);

    if body.text.len() < 1 || body.text.len() > 4096 {
        return Ok(Response::bad_request());
    }

    let result = sqlx::query("UPDATE tweets SET text = $1 WHERE id = $2 AND user_id = (SELECT user_id FROM sessions WHERE id = $3) RETURNING like_count, time_created")
        .bind(&body.text)
        .bind(id)
        .bind(session_id)
        .fetch_optional(&database)
        .await;

    match unwrap_internal_error!(result) {
        Some(row) => {
            let like_count = row.get_unchecked(0);
            let time_created = row.get_unchecked(1);
            Ok((
                StatusCode::OK,
                Response::success(Tweet::new(id, body.text, like_count, time_created)),
            ))
        }
        None => Ok(Response::not_found()),
    }
}
