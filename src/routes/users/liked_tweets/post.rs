use crate::common::*;

pub async fn post(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let session_id = check_auth_token!(request);

    #[derive(serde::Deserialize)]
    struct Body {
        tweet_id: i64,
    }

    let body = body!(request, Body);

    let result = sqlx::query(
        "INSERT INTO user_liked_tweets (user_id, tweet_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(session_id)
    .bind(body.tweet_id)
    .execute(&database)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Ok((StatusCode::CONFLICT, Response::error("Tweet already liked")))
            } else {
                Ok((StatusCode::CREATED, Response::empty()))
            }
        }
        Err(e) => {
            if let Some(e) = e.as_database_error() {
                if let Some(code) = e.code() {
                    // FOREIGN KEY VIOLATION
                    if code == "23503" {
                        return Ok(Response::not_found());
                    }
                }
            }

            log::error!("{:?}", e);

            Ok(Response::internal_server_error())
        }
    }
}
