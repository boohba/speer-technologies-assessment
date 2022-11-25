use crate::common::*;

pub async fn delete(request: &mut Request, database: Database) -> Result {
    check_content_type!(request);

    let session_id = check_auth_token!(request);

    #[derive(serde::Deserialize)]
    struct Body {
        tweet_id: i64,
    }

    let body = body!(request, Body);

    let result = sqlx::query("DELETE FROM user_liked_tweets WHERE user_id = $1 AND tweet_id = $2")
        .bind(session_id)
        .bind(body.tweet_id)
        .execute(&database)
        .await;

    if unwrap_internal_error!(result).rows_affected() == 0 {
        Ok(Response::not_found())
    } else {
        Ok((StatusCode::OK, Response::empty()))
    }
}
