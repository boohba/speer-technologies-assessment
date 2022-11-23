use once_cell::sync::{Lazy, OnceCell};

#[tokio::test(flavor = "current_thread")]
async fn main() {
    test_404().await;
    test_users().await;
    test_sessions().await;
}

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
});

static TOKEN: OnceCell<String> = OnceCell::new();

// might not be the most readable way to write tests but it works really well in a time-constrained environment
macro_rules! test {
    ($method:ident, $url:expr, $status:ident, $response:literal, $body:literal $(, $key:ident: $value:expr)*) => {
        let response = CLIENT.$method($url)
            $(.header(reqwest::header::$key, $value))*
            .body($body)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::$status);

        let body = response.bytes().await.unwrap();

        assert_eq!($response, std::str::from_utf8(&body).unwrap());
    }
}

async fn test_404() {
    println!("test_404");

    const URL: &str = "https://localhost:8443";

    test!(get, URL, NOT_FOUND, r#"{"error":true,"message":"Not Found"}"#, "");
}

async fn test_users() {
    println!("test_users");

    const URL: &str = "https://localhost:8443/users";

    test!(get, URL, METHOD_NOT_ALLOWED, r#"{"error":true,"message":"Method Not Allowed"}"#, "");

    test_users_post().await;
}

async fn test_users_post() {
    println!("test_users_post");

    const URL: &str = "https://localhost:8443/users";

    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Content-Type is not set"}"#, "");
    test!(post, URL, UNSUPPORTED_MEDIA_TYPE, r#"{"error":true,"message":"Unsupported Media Type"}"#, "", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, "aaa", CONTENT_TYPE: "application/json");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, r#"{"username":"aa","password":"aaa"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, r#"{"username":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","password":"aaa"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, CREATED, r#"{"error":false,"result":1}"#, r#"{"username":"hello","password":"world"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, CONFLICT, r#"{"error":true,"message":"Username already exists"}"#, r#"{"username":"hello","password":"world"}"#, CONTENT_TYPE: "application/json");
}

async fn test_sessions() {
    println!("test_sessions");

    const URL: &str = "https://localhost:8443/sessions";

    test!(get, URL, METHOD_NOT_ALLOWED, r#"{"error":true,"message":"Method Not Allowed"}"#, "");

    test_sessions_post().await;
}

async fn test_sessions_post() {
    println!("test_sessions_post");

    const URL: &str = "https://localhost:8443/sessions";

    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Content-Type is not set"}"#, "");
    test!(post, URL, UNSUPPORTED_MEDIA_TYPE, r#"{"error":true,"message":"Unsupported Media Type"}"#, "", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, "aaa", CONTENT_TYPE: "application/json");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, r#"{"username":"aa","password":"aaa"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, r#"{"username":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","password":"aaa"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, NOT_FOUND, r#"{"error":true,"message":"Not Found"}"#, r#"{"username":"foo","password":"bar"}"#, CONTENT_TYPE: "application/json");
    test!(post, URL, UNAUTHORIZED, r#"{"error":true,"message":"Unauthorized"}"#, r#"{"username":"hello","password":"bar"}"#, CONTENT_TYPE: "application/json");

    let response = CLIENT
        .post(URL)
        .body(r#"{"username":"hello","password":"world"}"#)
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    #[derive(serde::Deserialize)]
    struct Response {
        error: bool,
        result: String,
    }

    let response = serde_json::from_slice::<Response>(&response.bytes().await.unwrap()).unwrap();

    assert_eq!(response.error, false);

    TOKEN.set(response.result).unwrap();
}
