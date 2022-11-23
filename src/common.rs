pub use crate::*;

#[derive(serde::Serialize)]
pub struct Response<T: serde::Serialize> {
    error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
}

impl<T: serde::Serialize> Response<T> {
    #[inline(always)]
    pub fn success(result: T) -> Self {
        Response {
            error: false,
            message: None,
            result: Some(result),
        }
    }
}

impl Response<()> {
    pub const NOT_FOUND: Self = Self::failure("Not Found");
    pub const BAD_REQUEST: Self = Self::failure("Bad Request");
    pub const INTERNAL_SERVER_ERROR: Self = Self::failure("Internal Server Error");
    pub const UNAUTHORIZED: Self = Self::failure("Unauthorized");
    pub const UNSUPPORTED_MEDIA_TYPE: Self = Self::failure("Unsupported Media Type");

    #[inline(always)]
    pub const fn empty() -> Self {
        Response {
            error: false,
            message: None,
            result: None,
        }
    }

    #[inline(always)]
    pub const fn failure(message: &'static str) -> Self {
        Response::<()> {
            error: true,
            message: Some(message),
            result: None,
        }
    }
}

pub type Request = http::Request<h2::RecvStream>;
pub type Respond = h2::server::SendResponse<bytes::Bytes>;
pub type Database = sqlx::Pool<sqlx::postgres::Postgres>;

#[derive(serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        self.username.len() < 3
            || self.username.len() > 32
            || self.password.len() < 3
            || self.password.len() > 128
    }
}

pub static ARGON2: once_cell::sync::Lazy<argon2::Argon2> =
    once_cell::sync::Lazy::new(|| argon2::Argon2::default());

#[macro_export]
macro_rules! send_response {
    ($respond:ident, $code:ident, $body:expr) => {
        let response = http::Response::builder()
            .status(http::status::StatusCode::$code)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(())
            .unwrap();

        $respond.send_response(response, false)?.send_data(
            bytes::Bytes::from(serde_json::to_vec(&$body).unwrap()),
            true,
        )?;

        return Ok(());
    };
}

#[macro_export]
macro_rules! check_content_type {
    ($request:ident, $respond:ident) => {
        match $request.headers().get(http::header::CONTENT_TYPE) {
            Some(val) => {
                if val != "application/json" {
                    send_response!(
                        $respond,
                        UNSUPPORTED_MEDIA_TYPE,
                        Response::UNSUPPORTED_MEDIA_TYPE
                    );
                }
            }
            None => {
                send_response!($respond, BAD_REQUEST, Response::BAD_REQUEST);
            }
        }
    };
}

#[macro_export]
macro_rules! body {
    ($request:ident, $respond:ident, $type:ty) => {{
        // the payload must be small enough to fit in one frame (don't do that in production)
        let body = match $request.body_mut().data().await {
            Some(body) => body?,
            None => {
                send_response!($respond, BAD_REQUEST, Response::BAD_REQUEST);
            }
        };

        match serde_json::from_slice::<$type>(&body) {
            Ok(body) => body,
            Err(_) => {
                send_response!($respond, BAD_REQUEST, Response::BAD_REQUEST);
            }
        }
    }};
}

#[macro_export]
macro_rules! unwrap_internal_error {
    ($respond:ident, $result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                log::error!("{:?}", e);

                send_response!(
                    $respond,
                    INTERNAL_SERVER_ERROR,
                    Response::INTERNAL_SERVER_ERROR
                );
            }
        }
    };
}

#[macro_export]
macro_rules! check_auth_token {
    ($request:ident, $respond:ident) => {{
        match $request.headers().get(http::header::AUTHORIZATION) {
            Some(token) => match auth::decode_token(token.as_bytes()) {
                Ok(session_id) => session_id,
                Err(_) => {
                    send_response!($respond, UNAUTHORIZED, Response::UNAUTHORIZED);
                }
            },
            None => {
                send_response!($respond, UNAUTHORIZED, Response::UNAUTHORIZED);
            }
        }
    }};
}
