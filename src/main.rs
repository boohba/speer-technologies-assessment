mod routes;

pub mod common;

pub mod prelude {
    pub use crate::common::*;
    pub use crate::*;
}

use prelude::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let tls_acceptor = {
        let cert = rustls::Certificate(include_bytes!("../tls/cert.der").to_vec());
        let pkey = rustls::PrivateKey(include_bytes!("../tls/pkey.der").to_vec());

        let mut config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], pkey)
            .unwrap();

        config.alpn_protocols = vec![b"h2".to_vec()];

        tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config))
    };

    let database_address = std::env::var("DATABASE_ADDRESS").unwrap_or(String::from(
        "postgres://postgres:postgres@localhost/postgres",
    ));

    // https://wiki.postgresql.org/wiki/Number_Of_Database_Connections
    let database = sqlx::postgres::PgPoolOptions::new()
        .max_connections((num_cpus::get_physical() * 2 + 1) as u32)
        .connect(&database_address)
        .await
        .unwrap();

    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or(String::from("[::]:8443"));

    // setup a socket for accepting connections
    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();

    log::info!("Listening on {}", listener.local_addr().unwrap());

    loop {
        // try to accept a connection
        match listener.accept().await {
            Ok((connection, _)) => {
                // spawn a task to asynchronously handle the connection
                tokio::spawn(handle_connection(
                    connection,
                    tls_acceptor.clone(),
                    database.clone(),
                ));
            }
            Err(e) => {
                log::warn!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    connection: tokio::net::TcpStream,
    tls_acceptor: tokio_rustls::TlsAcceptor,
    database: Database,
) {
    // try to perform a TLS handshake
    match tls_acceptor.accept(connection).await {
        Ok(connection) => {
            // try to perform an HTTP/2 handshake
            match h2::server::handshake(connection).await {
                Ok(mut connection) => {
                    // try to accept an HTTP/2 request
                    while let Some(result) = connection.accept().await {
                        match result {
                            Ok((request, respond)) => {
                                // spawn a task to asynchronously handle the request
                                tokio::spawn(handle_request(request, respond, database.clone()));
                            }
                            Err(e) => {
                                if let Some(e) = e.get_io() {
                                    if matches!(e.kind(), std::io::ErrorKind::UnexpectedEof) {
                                        return; // connection is closed, this is not an error
                                    }
                                }

                                log::warn!("Failed to accept an HTTP/2 request: {}", e);

                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to perform an HTTP/2 handshake: {}", e);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to perform a TLS handshake: {}", e);
        }
    }
}

#[derive(serde::Serialize)]
struct Response<T: serde::Serialize> {
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
    #[inline(always)]
    pub fn failure(message: &'static str) -> Self {
        Response::<()> {
            error: true,
            message: Some(message),
            result: None,
        }
    }
}

async fn handle_request(mut request: Request, mut respond: Respond, database: Database) {
    macro_rules! error {
        ($code:ident) => {{
            let status = http::status::StatusCode::$code;

            let response = http::Response::builder()
                .status(status)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(())
                .unwrap();

            match respond.send_response(response, false) {
                Ok(mut send) => {
                    let response = Response::failure(status.canonical_reason().unwrap());

                    send.send_data(
                        bytes::Bytes::from(serde_json::to_vec(&response).unwrap()),
                        true,
                    )
                }
                Err(e) => Err(e),
            }
        }};
    }

    macro_rules! call {
        ($handler:path) => {
            $handler(&mut request, &mut respond, database).await
        };
    }

    let result = match request.uri().path() {
        "/users" => match *request.method() {
            http::Method::POST => call!(routes::users::post),
            _ => error!(METHOD_NOT_ALLOWED),
        },
        "/sessions" => match *request.method() {
            http::Method::POST => call!(routes::sessions::post),
            _ => error!(METHOD_NOT_ALLOWED),
        },
        _ => error!(NOT_FOUND),
    };

    if let Err(e) = result {
        log::warn!("Failed to handle HTTP/2 request: {}", e);
    }
}

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
    ($request:ident, $respond:ident, $content_type:literal) => {
        match $request.headers().get(http::header::CONTENT_TYPE) {
            Some(val) => {
                if val != $content_type {
                    send_response!(
                        $respond,
                        UNSUPPORTED_MEDIA_TYPE,
                        Response::failure("Unsupported Media Type")
                    );
                }
            }
            None => {
                send_response!(
                    $respond,
                    BAD_REQUEST,
                    Response::failure("Content-Type is not set")
                );
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
                send_response!(
                    $respond,
                    BAD_REQUEST,
                    Response::failure("Invalid request payload")
                );
            }
        };

        match serde_json::from_slice::<$type>(&body) {
            Ok(body) => body,
            Err(_) => {
                send_response!(
                    $respond,
                    BAD_REQUEST,
                    Response::failure("Invalid request payload")
                );
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
                    Response::failure("Internal Server Error")
                );
            }
        }
    };
}
