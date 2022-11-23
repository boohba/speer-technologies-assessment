use hmac::{Hmac, Mac};
use sha2::Sha256;

static AUTH_SECRET: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| std::env::var("AUTH_SECRET").unwrap_or(String::from("secret")));

#[macro_export]
macro_rules! signature {
    ($session_id:expr) => {{
        let mut mac = Hmac::<Sha256>::new_from_slice(AUTH_SECRET.as_bytes()).unwrap();
        mac.update($session_id);
        mac.finalize().into_bytes()
    }};
}

#[inline(always)]
pub fn create_token(session_id: &[u8]) -> String {
    // session_id + hmacsha256 signature, kind of like JWT but more efficient
    let mut token = [0u8; 40];

    unsafe {
        let signature = signature!(session_id);
        std::ptr::copy_nonoverlapping(session_id.as_ptr(), token.as_mut_ptr(), 8);
        std::ptr::copy_nonoverlapping(signature.as_ptr(), token.as_mut_ptr().offset(8), 32);
    }

    base64::encode(token)
}

#[inline(always)]
pub fn decode_token(token: &[u8]) -> Result<i64, ()> {
    let token = match base64::decode(token) {
        Ok(token) => token,
        Err(_) => {
            return Err(());
        }
    };

    if token.len() != 40 {
        return Err(());
    }

    let (session_id, signature) = token.split_at(8);

    let valid_signature = signature!(session_id);

    for index in 0..32 {
        if signature[index] != valid_signature[index] {
            return Err(());
        }
    }

    let mut buf = [0; 8];

    unsafe { std::ptr::copy_nonoverlapping(session_id.as_ptr(), buf.as_mut_ptr(), 8) }

    Ok(i64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_and_decode() {
        let mut token = crate::auth::create_token(&[0; 8]);

        let session_id = crate::auth::decode_token(token.as_bytes());
        assert!(matches!(Ok::<i64, ()>(0), session_id));

        unsafe { std::ptr::copy_nonoverlapping([0u8; 1].as_ptr(), token.as_mut_ptr(), 1) }

        let session_id = crate::auth::decode_token(token.as_bytes());
        assert!(matches!(Err::<i64, ()>(()), session_id));
    }
}
