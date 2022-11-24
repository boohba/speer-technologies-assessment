use crate::common::*;

pub async fn post(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let session_id = check_auth_token!(request);

    #[derive(serde::Deserialize)]
    pub struct Body {
        text: String,
    }

    let body = body!(request, Body);

    if body.text.len() < 1 || body.text.len() > 4096 {
        return Ok(Response::bad_request());
    }

    // this is still relatively efficient (in case you are wondering)
    let result = sqlx::query("INSERT INTO tweets (user_id, text) VALUES ((SELECT user_id FROM sessions WHERE id = $1), $2) RETURNING id, time_created")
        .bind(session_id)
        .bind(&body.text)
        .fetch_one(&database)
        .await;

    let row = unwrap_internal_error!(result);
    let id = row.get_unchecked::<i64, _>(0);
    let time_created = row.get_unchecked::<i64, _>(1);

    Ok((
        StatusCode::CREATED,
        Response::success(Tweet::new(id, body.text, 0, time_created)),
    ))
}
