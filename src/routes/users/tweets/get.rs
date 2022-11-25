use crate::common::*;

pub async fn get(request: &mut Request, database: Database) -> Result {
    let session_id = check_auth_token!(request);

    let mut limit = 50;
    let mut offset = 0;

    // why we still here?
    if let Some(query) = request.uri().query() {
        for pair in query.split("&").take(2) {
            if let Some((key, value)) = pair.split_once("=") {
                match key {
                    "limit" => {
                        if let Ok(value) = value.parse::<i32>() {
                            if value < 0 || value > 50 {
                                return Ok(Response::bad_request());
                            } else {
                                limit = value;
                            }
                        } else {
                            return Ok(Response::bad_request());
                        }
                    }
                    "offset" => {
                        if let Ok(value) = value.parse::<i32>() {
                            if value < 0 {
                                return Ok(Response::bad_request());
                            } else {
                                offset = value;
                            }
                        } else {
                            return Ok(Response::bad_request());
                        }
                    }
                    _ => {
                        return Ok(Response::bad_request());
                    }
                }
            } else {
                return Ok(Response::bad_request());
            }
        }
    }

    if limit < 0 || limit > 50 || offset < 0 {
        return Ok(Response::bad_request());
    }

    let result = sqlx::query("SELECT id, text, like_count, time_created FROM tweets WHERE user_id = (SELECT user_id FROM sessions WHERE id = $1) ORDER BY time_created DESC LIMIT $2 OFFSET $3")
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&database)
        .await;

    let response = unwrap_internal_error!(result)
        .into_iter()
        .map(|row| {
            Tweet::new(
                row.get_unchecked(0),
                row.get_unchecked(1),
                row.get_unchecked(2),
                row.get_unchecked(3),
            )
        })
        .collect::<Vec<Tweet>>();

    Ok((StatusCode::OK, Response::success(response)))
}
