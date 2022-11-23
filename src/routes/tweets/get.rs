use crate::common::*;

pub async fn get(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    let session_id = check_auth_token!(request, respond);

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
                                send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
                            } else {
                                limit = value;
                            }
                        } else {
                            send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
                        }
                    }
                    "offset" => {
                        if let Ok(value) = value.parse::<i32>() {
                            if value < 0 {
                                send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
                            } else {
                                offset = value;
                            }
                        } else {
                            send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
                        }
                    }
                    _ => {
                        send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
                    }
                }
            } else {
                send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
            }
        }
    }

    if limit < 0 || limit > 50 || offset < 0 {
        send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
    }

    // making an index on tweets.user_id and/or tweets.time_created will not necessarily improve
    // performance, or at the very least, will be a preemptive optimization.
    let result = sqlx::query("SELECT id, text, time_created FROM tweets WHERE user_id = (SELECT user_id FROM sessions WHERE id = $1) ORDER BY time_created DESC LIMIT $2 OFFSET $3")
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&database)
        .await;

    #[derive(serde::Serialize)]
    struct Tweet {
        id: i64,
        text: String,
        time_created: i64,
    }

    use sqlx::Row;

    let response = unwrap_internal_error!(respond, result)
        .into_iter()
        .map(|row| Tweet {
            id: row.get_unchecked(0),
            text: row.get_unchecked(1),
            time_created: row.get_unchecked(2),
        })
        .collect::<Vec<Tweet>>();

    send_response!(respond, OK, Response::success(response));
}
