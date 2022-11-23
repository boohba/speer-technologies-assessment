use crate::common::*;

pub async fn patch(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    check_content_type!(request, respond);

    let session_id = check_auth_token!(request, respond);

    #[derive(serde::Deserialize)]
    pub struct Tweet {
        id: i64,
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

    let result = sqlx::query("UPDATE tweets SET text = $1 WHERE id = $2 AND user_id = (SELECT user_id FROM sessions WHERE id = $3)")
        .bind(tweet.text)
        .bind(tweet.id)
        .bind(session_id)
        .execute(&database)
        .await;

    if unwrap_internal_error!(respond, result).rows_affected() == 0 {
        send_response!(respond, NOT_FOUND, Response::NOT_FOUND);
    }

    send_response!(respond, OK, Response::empty());
}
