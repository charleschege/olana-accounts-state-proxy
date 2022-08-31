use crate::{JsonError, RpcProxyError, RpcProxyResult, RpcRequest};
use hyper::{Body, Method, Request, Response, StatusCode};
use jsonrpsee::core::middleware::{Headers, HttpMiddleware, MethodKind, Params};
use std::{net::SocketAddr, time::Instant};

pub(crate) async fn processor(req: Request<Body>) -> RpcProxyResult<Response<Body>> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try `POST` method to `/`");
        }
        (&Method::POST, "/") => {
            let body = hyper::body::to_bytes(req).await?;
            let body = String::from_utf8_lossy(&body).to_string();
            let body = body.trim();

            match parse_body(body, &mut response) {
                Ok(_) => (),
                Err(error) => match error {
                    RpcProxyError::SerdeJsonError(_) => (),
                    _ => return Err(error),
                },
            }
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

/// The handler for incoming JSON POST requests
#[derive(Debug, Clone)]
pub struct RpcProxyMiddleware;

impl HttpMiddleware for RpcProxyMiddleware {
    type Instant = Instant;

    // Called once the HTTP request is received, it may be a single JSON-RPC call
    // or batch.
    fn on_request(&self, _remote_addr: SocketAddr, _headers: &Headers) -> Instant {
        Instant::now()
    }

    // Called once a single JSON-RPC method call is processed, it may be called multiple times
    // on batches.
    fn on_call(&self, method_name: &str, params: Params, kind: MethodKind) {
        println!(
            "Call to method: '{}' params: {:?}, kind: {}",
            method_name, params, kind
        );
    }

    // Called once a single JSON-RPC call is completed, it may be called multiple times
    // on batches.
    fn on_result(&self, method_name: &str, success: bool, started_at: Instant) {
        println!("Call to '{}' took {:?}", method_name, started_at.elapsed());
    }

    // Called the entire JSON-RPC is completed, called on once for both single calls or batches.
    fn on_response(&self, result: &str, started_at: Instant) {
        println!(
            "complete JSON-RPC response: {}, took: {:?}",
            result,
            started_at.elapsed()
        );
    }
}
