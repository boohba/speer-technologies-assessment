use crate::common::*;

pub async fn delete(
    request: &mut Request,
    respond: &mut Respond,
    database: Database,
) -> Result<(), h2::Error> {
    check_content_type!(request, respond);

    let session_id = check_auth_token!(request, respond);

    #[derive(serde::Deserialize)]
    pub struct Tweet {
        id: i64,
    }

    let result = sqlx::query("DELETE FROM tweets WHERE id = $1 AND user_id = (SELECT user_id FROM sessions WHERE id = $2)")
        .bind(body!(request, respond, Tweet).id)
        .bind(session_id)
        .execute(&database)
        .await;

    if unwrap_internal_error!(respond, result).rows_affected() == 0 {
        send_response!(respond, NOT_FOUND, Response::NOT_FOUND);
    }

    send_response!(respond, OK, Response::empty());
}
