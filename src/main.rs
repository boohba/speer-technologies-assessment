mod common;
mod routes;

pub mod auth;

use common::*;

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

    let database_address = std::env::var("DATABASE_ADDRESS")
        .unwrap_or("postgres://postgres:postgres@localhost/postgres".to_string());

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

async fn handle_request(mut request: Request, mut respond: Respond, database: Database) {
    macro_rules! call {
        ($handler:path) => {
            $handler(&mut request, database).await
        };
    }

    let result = match request.uri().path() {
        "/users" => match *request.method() {
            http::Method::POST => call!(routes::users::post),
            _ => Ok(Response::method_not_allowed()),
        },
        "/users/@me/sessions" => match *request.method() {
            http::Method::POST => call!(routes::users::sessions::post),
            _ => Ok(Response::method_not_allowed()),
        },
        "/users/@me/tweets" => match *request.method() {
            http::Method::POST => call!(routes::users::tweets::post),
            http::Method::GET => call!(routes::users::tweets::get),
            _ => Ok(Response::method_not_allowed()),
        },
        "/users/@me/liked_tweets" => match *request.method() {
            http::Method::POST => call!(routes::users::liked_tweets::post),
            http::Method::DELETE => call!(routes::users::liked_tweets::delete),
            _ => Ok(Response::method_not_allowed()),
        },
        path => {
            use once_cell::sync::Lazy;
            use regex::Regex;

            static REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("^/users/@me/tweets/[0-9]{1,16}$").unwrap());

            if REGEX.is_match(path) {
                match *request.method() {
                    http::Method::PATCH => call!(routes::users::tweets::patch),
                    http::Method::DELETE => call!(routes::users::tweets::delete),
                    _ => Ok(Response::method_not_allowed()),
                }
            } else {
                Ok(Response::not_found())
            }
        }
    };

    match result {
        Ok((code, body)) => {
            let response = http::Response::builder()
                .status(code)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(())
                .unwrap();

            match respond.send_response(response, false) {
                Ok(mut send) => {
                    if let Err(e) = send.send_data(body, true) {
                        log::warn!("Failed to send HTTP/2 data frame: {}", e);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to send HTTP/2 response: {}", e);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to handle HTTP/2 request: {}", e);
        }
    }
}
