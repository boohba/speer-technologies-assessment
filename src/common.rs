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

pub static AUTH_SECRET: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| std::env::var("AUTH_SECRET").unwrap_or(String::from("secret")));

#[macro_export]
macro_rules! signature {
    ($session_id:expr) => {{
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(AUTH_SECRET.as_bytes()).unwrap();
        mac.update($session_id);
        mac.finalize().into_bytes()
    }};
}
