#![rustfmt::skip]
#![feature(custom_inner_attributes)]

use http::{header, StatusCode};
use once_cell::sync::{Lazy, OnceCell};
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn main() {
    macro_rules! run {
        ($ident:ident) => {
            $ident().await;
        };
    }

    run!(test_404);
    run!(test_405);
    run!(test_create_user);
    run!(test_create_session);
    run!(test_create_tweet);
    run!(test_get_tweets);
    run!(test_edit_tweet);
    run!(test_delete_tweet);
    run!(test_like_tweet);
    run!(test_unlike_tweet);
}

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
});

static TOKEN: OnceCell<String> = OnceCell::new();

const SERVER: &str = "https://localhost:8443";

#[derive(Eq, PartialEq, serde::Deserialize)]
struct Response<T> {
    error: bool,
    message: Option<String>,
    result: Option<T>,
}

#[derive(Eq, PartialEq, serde::Deserialize)]
struct User {
    id: i64,
    username: String,
}

#[derive(Eq, PartialEq, serde::Deserialize)]
struct Tweet {
    id: i64,
    text: String,
    like_count: i32,
    time_created: i64,
}

async fn test_404() {
    println!("test_404");

    let response = CLIENT.put(format!("{}/something", SERVER))
        .send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn test_405() {
    println!("test_405");

    let response = CLIENT.put(format!("{}/users", SERVER))
        .send().await.unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

async fn assert_success<T: DeserializeOwned>(status: StatusCode, mut request: RequestBuilder) -> Option<T> {
    if let Some(token) = TOKEN.get() {
        request = request.header(header::AUTHORIZATION, token);
    }

    let response = request.send().await.unwrap();

    assert_eq!(response.status(), status);

    let response = response.json::<Response<T>>().await.unwrap();

    assert_eq!(response.error, false);

    response.result
}

async fn assert_error<T: DeserializeOwned>(status: StatusCode, mut request: RequestBuilder) {
    if let Some(token) = TOKEN.get() {
        request = request.header(header::AUTHORIZATION, token);
    }

    let response = request
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), status);

    let response = response.json::<Response<T>>().await.unwrap();

    assert_eq!(response.error, true);
    assert!(matches!(response.message, Some(..)));
    assert!(matches!(response.result, None));
}

async fn assert_unauthorized(request: RequestBuilder) {
    let response = request
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
}

macro_rules! boilerplate {
    ($url:ident, $method:ident, $ty:ty) => {
        assert_error::<$ty>(StatusCode::UNSUPPORTED_MEDIA_TYPE, CLIENT.$method($url)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")).await;

        assert_error::<$ty>(StatusCode::BAD_REQUEST, CLIENT.$method($url)
            .header(header::CONTENT_TYPE, "application/json")
            .body("aaa")).await;
    }
}

async fn test_create_user() {
    println!("test_create_user");

    let url = &format!("{}/users", SERVER);

    boilerplate!(url, post, User);

    assert_error::<User>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "username": "a", "password": "a" }))).await;

    assert_error::<User>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "username": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "password": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }))).await;

    let response = assert_success::<User>(StatusCode::CREATED, CLIENT.post(url)
        .json(&json!({ "username": "hello", "password": "world" }))).await.unwrap();

    assert_eq!(response.username, "hello");

    assert_error::<User>(StatusCode::CONFLICT, CLIENT.post(url)
        .json(&json!({ "username": "hello", "password": "world" }))).await;
}

async fn test_create_session() {
    println!("test_create_session");

    let url = &format!("{}/users/@me/sessions", SERVER);

    boilerplate!(url, post, String);

    assert_error::<String>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "username": "a", "password": "a" }))).await;

    assert_error::<String>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "username": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "password": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }))).await;

    assert_error::<String>(StatusCode::NOT_FOUND, CLIENT.post(url)
        .json(&json!({ "username": "helo", "password": "world" }))).await;

    assert_error::<String>(StatusCode::NOT_FOUND, CLIENT.post(url)
        .json(&json!({ "username": "hello", "password": "wowld" }))).await;

    let response = assert_success::<String>(StatusCode::CREATED, CLIENT.post(url)
        .json(&json!({ "username": "hello", "password": "world" }))).await.unwrap();

    TOKEN.set(response).unwrap();
}

async fn test_create_tweet() {
    println!("test_create_tweet");

    let url = &format!("{}/users/@me/tweets", SERVER);

    boilerplate!(url, post, Tweet);

    assert_unauthorized(CLIENT.post(url)).await;

    assert_error::<Tweet>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "text": "" }))).await;

    assert_error::<Tweet>(StatusCode::BAD_REQUEST, CLIENT.post(url)
        .json(&json!({ "text": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }))).await;

    for id in 1..3 {
        let response = assert_success::<Tweet>(StatusCode::CREATED, CLIENT.post(url)
            .json(&json!({ "text": "hello" }))).await.unwrap();

        assert_eq!(response.id, id);
        assert_eq!(response.text, "hello");
        assert_eq!(response.like_count, 0);
    }
}

async fn test_get_tweets() {
    println!("test_get_tweets");

    let url = &format!("{}/users/@me/tweets", SERVER);

    assert_unauthorized(CLIENT.get(url)).await;

    assert_error::<Vec<Tweet>>(StatusCode::BAD_REQUEST, CLIENT.get(url)
        .query(&[("limit", "500"), ("offset", "-1")]))
        .await;

    for limit in 0..3 {
        let response = assert_success::<Vec<Tweet>>(StatusCode::OK, CLIENT.get(url)
            .query(&[("limit", limit.to_string())]))
            .await.unwrap();

        assert_eq!(response.len(), limit);
    }

    for offset in 0..2 {
        let response = assert_success::<Vec<Tweet>>(StatusCode::OK, CLIENT.get(url)
            .query(&[("limit", "1"), ("offset", &offset.to_string())]))
            .await.unwrap();

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, offset + 1);
    }
}

async fn test_edit_tweet() {
    println!("test_edit_tweet");

    let url = &format!("{}/users/@me/tweets/1", SERVER);

    boilerplate!(url, patch, Tweet);

    assert_unauthorized(CLIENT.patch(url)).await;

    assert_error::<Tweet>(StatusCode::BAD_REQUEST, CLIENT.patch(url)
        .json(&json!({ "text": "" }))).await;

    assert_error::<Tweet>(StatusCode::BAD_REQUEST, CLIENT.patch(url)
        .json(&json!({ "text": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }))).await;

    let response = assert_success::<Tweet>(StatusCode::OK, CLIENT.patch(url)
        .json(&json!({ "text": "Hello, World!" }))).await.unwrap();

    assert_eq!(response.id, 1);
    assert_eq!(response.text, "Hello, World!");
}

async fn test_delete_tweet() {
    println!("test_delete_tweet");

    let url = &format!("{}/users/@me/tweets/1", SERVER);

    assert_unauthorized(CLIENT.delete(url)).await;
    assert_success::<()>(StatusCode::OK, CLIENT.delete(url)).await;
    assert_error::<()>(StatusCode::NOT_FOUND, CLIENT.delete(url)).await;
}

async fn test_like_tweet() {
    println!("test_like_tweet");

    let url = &format!("{}/users/@me/liked_tweets", SERVER);

    boilerplate!(url, post, ());

    assert_error::<()>(StatusCode::NOT_FOUND, CLIENT.post(url)
        .json(&json!({ "tweet_id": 1 }))).await;

    assert_success::<()>(StatusCode::CREATED, CLIENT.post(url)
        .json(&json!({ "tweet_id": 2 }))).await;

    let response = assert_success::<Vec<Tweet>>(StatusCode::OK, CLIENT.get(format!("{}/users/@me/tweets", SERVER)))
        .await.unwrap();

    assert_eq!(response.len(), 1);
    assert_eq!(response[0].like_count, 1);
}

async fn test_unlike_tweet() {
    println!("test_unlike_tweet");

    let url = &format!("{}/users/@me/liked_tweets", SERVER);

    boilerplate!(url, delete, ());

    assert_error::<()>(StatusCode::NOT_FOUND, CLIENT.delete(url)
        .json(&json!({ "tweet_id": 1 }))).await;

    assert_success::<()>(StatusCode::OK, CLIENT.delete(url)
        .json(&json!({ "tweet_id": 2 }))).await;

    let response = assert_success::<Vec<Tweet>>(StatusCode::OK, CLIENT.get(format!("{}/users/@me/tweets", SERVER)))
        .await.unwrap();

    assert_eq!(response.len(), 1);
    assert_eq!(response[0].like_count, 0);
}