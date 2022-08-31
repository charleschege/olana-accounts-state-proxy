use crate::{JsonError, RpcProxyError, RpcProxyResult, RpcRequest};
use hyper::{Body, Method, Request, Response, StatusCode};

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
