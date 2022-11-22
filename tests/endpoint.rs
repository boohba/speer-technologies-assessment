#[tokio::test]
async fn main() {
    let mut tasks = Vec::<tokio::task::JoinHandle<()>>::new();

    macro_rules! run {
        ($test:ident) => {
            tasks.push(tokio::spawn($test()));
        };
    }

    run!(test_404);
    run!(test_users);

    for task in tasks {
        task.await.unwrap();
    }
}

static CLIENT: once_cell::sync::Lazy<reqwest::Client> = once_cell::sync::Lazy::new(|| {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
});

// might not be the most readable way to write tests but it works really well in a time-constrained environment
macro_rules! test {
    ($method:ident, $url:ident, $status:ident, $response:literal, $body:literal $(, $key:ident: $value:literal)*) => {
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

    test!(get,URL,NOT_FOUND,r#"{"error":true,"message":"Not Found"}"#,"");
}

async fn test_users() {
    println!("test_users");

    const URL: &str = "https://localhost:8443/users";

    test!(get,URL,METHOD_NOT_ALLOWED,r#"{"error":true,"message":"Method Not Allowed"}"#,"");
    test!(put,URL,BAD_REQUEST,r#"{"error":true,"message":"Content-Type is not set"}"#,"");
    test!(put, URL, UNSUPPORTED_MEDIA_TYPE, r#"{"error":true,"message":"Unsupported Media Type"}"#, "", CONTENT_TYPE: "application/json");
    test!(put, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, "aaa", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(put, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, "username=a&password=a", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(put, URL, BAD_REQUEST, r#"{"error":true,"message":"Invalid request payload"}"#, "username=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&password=aaa", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(put, URL, CREATED, r#"{"error":false}"#, "username=hello&password=world", CONTENT_TYPE: "application/x-www-form-urlencoded");
    test!(put, URL, CONFLICT, r#"{"error":true,"message":"Username already exists"}"#, "username=hello&password=world", CONTENT_TYPE: "application/x-www-form-urlencoded");
}
