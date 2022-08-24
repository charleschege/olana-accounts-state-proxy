#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use hyper::{
    service::{make_service_fn, service_fn},
    Method, StatusCode,
};
use hyper::{Body, Request, Response, Server};
use std::net::SocketAddr;

mod requests;
pub use requests::*;

mod errors;
pub use errors::*;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 1024));
    println!("Listening at port: `0.0.0.0:1024`",);

    let make_svc = make_service_fn(|_conn| async { Ok::<_, RpcProxyError>(service_fn(processor)) });

    let server = Server::bind(&addr).serve(make_svc);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e); //TODO Log to facade
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn processor(req: Request<Body>) -> RpcProxyResult<Response<Body>> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try `POST` method to `/getAccountInfo`");
        }
        (&Method::POST, "/") => {
            *response.body_mut() = Body::from("Try `POST` method to `/getAccountInfo`");
        }
        (&Method::POST, "/getAccountInfo") => {
            let body = hyper::body::to_bytes(req).await?;
            let body = String::from_utf8_lossy(&body).to_string();
            let body = body.trim();

            parse_body(body, &mut response)?;
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(response)
}

/// Parses bodies of all `POST` requests
pub fn parse_body(body: &str, response: &mut Response<Body>) -> RpcProxyResult<RpcRequest> {
    let jd = &mut serde_json::Deserializer::from_str(body);

    let deser_body: Result<RpcRequest, _> = serde_path_to_error::deserialize(jd);
    match deser_body {
        Ok(rpc_request) => {
            if rpc_request.parameter_checks(response)? {
                rpc_request.respond(response)?;
            } else {
            }

            Ok(rpc_request)
        }
        Err(error) => {
            let path = error.to_string();

            JsonError::new()
                .add_message("Unable to parse the JSON request")
                .add_data(&path)
                .response(response)?;

            Err(RpcProxyError::SerdeJsonError(path))
        }
    }
}
