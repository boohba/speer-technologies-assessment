use crate::prelude::*;

pub async fn post(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    check_content_type!(request, respond, "application/json");

    let session_id = check_auth_token!(request, respond);

    #[derive(serde::Deserialize)]
    pub struct Tweet {
        text: String,
    }

    impl Tweet {
        #[inline(always)]
        pub fn is_invalid(&self) -> bool {
            self.text.len() < 1 || self.text.len() > 4096
        }
    }

    let tweet = body!(request, respond, Tweet);

    if tweet.is_invalid() {
        send_response!(respond, BAD_REQUEST, Response::BAD_REQUEST);
    }

    // this is still relatively efficient (in case you are wondering)
    let result = sqlx::query("INSERT INTO tweets (user_id, text) VALUES ((SELECT user_id FROM sessions WHERE id = $1), $2) RETURNING id")
        .bind(session_id)
        .bind(tweet.text)
        .fetch_one(&database)
        .await;

    use sqlx::Row;

    let tweet_id = unwrap_internal_error!(respond, result).get_unchecked::<i64, _>(0);

    send_response!(respond, CREATED, Response::success(tweet_id));
}
