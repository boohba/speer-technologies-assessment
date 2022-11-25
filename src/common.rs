use once_cell::sync::Lazy;

pub use crate::*;
pub use bytes::Bytes;
pub use http::StatusCode;
pub use serde_json::json;
pub use sqlx::Row;

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
    fn new(error: bool, message: Option<&'static str>, result: Option<T>) -> Bytes {
        Bytes::from(
            serde_json::to_vec(&Self {
                error,
                message,
                result,
            })
            .unwrap(),
        )
    }

    #[inline(always)]
    pub fn success(result: T) -> Bytes {
        Self::new(false, None, Some(result))
    }
}

macro_rules! error {
    ($ident:ident, $status:ident) => {
        #[inline(always)]
        pub fn $ident() -> (StatusCode, Bytes) {
            (
                StatusCode::$status,
                Response::<()>::error(StatusCode::$status.canonical_reason().unwrap()),
            )
        }
    };
}

impl Response<()> {
    #[inline(always)]
    pub fn error(message: &'static str) -> Bytes {
        Self::new(true, Some(message), None)
    }

    #[inline(always)]
    pub fn empty() -> Bytes { Self::new(false, None, None) }

    error!(not_found, NOT_FOUND);
    error!(bad_request, BAD_REQUEST);
    error!(internal_server_error, INTERNAL_SERVER_ERROR);
    error!(unauthorized, UNAUTHORIZED);
    error!(unsupported_media_type, UNSUPPORTED_MEDIA_TYPE);
    error!(method_not_allowed, METHOD_NOT_ALLOWED);
}

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

#[derive(serde::Serialize)]
pub struct Tweet {
    id: i64,
    text: String,
    like_count: i32,
    time_created: i64,
}

impl Tweet {
    #[inline(always)]
    pub fn new(id: i64, text: String, like_count: i32, time_created: i64) -> Self {
        Self {
            id,
            text,
            like_count,
            time_created,
        }
    }
}

pub type Result = std::result::Result<(StatusCode, Bytes), h2::Error>;
pub type Request = http::Request<h2::RecvStream>;
pub type Respond = h2::server::SendResponse<bytes::Bytes>;
pub type Database = sqlx::Pool<sqlx::postgres::Postgres>;

pub static ARGON2: Lazy<argon2::Argon2> = Lazy::new(|| argon2::Argon2::default());

#[macro_export]
macro_rules! check_content_type {
    ($request:ident) => {
        match $request.headers().get(http::header::CONTENT_TYPE) {
            Some(val) => {
                if val != "application/json" {
                    return Ok(Response::unsupported_media_type());
                }
            }
            None => {
                return Ok(Response::bad_request());
            }
        }
    };
}

#[macro_export]
macro_rules! body {
    ($request:ident, $type:ty) => {{
        // the payload must be small enough to fit in one frame (don't do that in production)
        let body = match $request.body_mut().data().await {
            Some(body) => body?,
            None => {
                return Ok(Response::bad_request());
            }
        };

        match serde_json::from_slice::<$type>(&body) {
            Ok(body) => body,
            Err(_) => {
                return Ok(Response::bad_request());
            }
        }
    }};
}

#[macro_export]
macro_rules! unwrap_internal_error {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                log::error!("{:?}", e);

                return Ok(Response::internal_server_error());
            }
        }
    };
}

#[macro_export]
macro_rules! check_auth_token {
    ($request:ident) => {{
        match $request.headers().get(http::header::AUTHORIZATION) {
            Some(token) => match auth::decode_token(token.as_bytes()) {
                Ok(session_id) => session_id,
                Err(_) => {
                    return Ok(Response::unauthorized());
                }
            },
            None => {
                return Ok(Response::unauthorized());
            }
        }
    }};
}

#[macro_export]
macro_rules! parse_path_var {
    ($request:ident, $ty:ty) => {
        match $request.uri().path().rsplit_once("/") {
            Some((_, val)) => match val.parse::<$ty>() {
                Ok(val) => val,
                Err(_) => {
                    return Ok(Response::bad_request());
                }
            },
            None => {
                unreachable!()
            }
        }
    };
}
