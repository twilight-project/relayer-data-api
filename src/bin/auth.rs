use http::{Request, StatusCode};
use hyper::{body::to_bytes, server::Server, Body, Response};
use log::debug;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::{make::Shared, ServiceBuilder};
use verify_keplr_sign::{verify_arbitrary, Signature};

#[derive(Deserialize)]
pub struct Auth {
    pub account_address: String,
    pub signature: Signature,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub account_address: String,
}

async fn handler(request: Request<Body>) -> Result<Response<Body>, http::Error> {
    debug!("Auth request");
    if request.uri() != "/login" {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty());
    }

    let request: Vec<u8> = to_bytes(request.into_body())
        .await
        .expect("Bad bytes")
        .to_vec();
    let Auth {
        account_address,
        signature,
        data,
    } = match serde_json::from_slice(&request) {
        Ok(auth) => auth,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty());
        }
    };

    let is_ok = verify_arbitrary(
        &account_address,
        &signature.pub_key.sig_value,
        data.as_bytes(),
        &signature,
    );

    if !is_ok {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty());
    }

    // TODO: check if user exists in DB!

    let token = AuthResponse { account_address };
    let response = serde_json::to_string(&token).expect("Could not serialize");

    return Response::builder()
        .status(StatusCode::OK)
        .body(response.into());
}

#[tokio::main]
async fn main() {
    let service = ServiceBuilder::new().service_fn(handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    Server::bind(&addr)
        .serve(Shared::new(service))
        .await
        .expect("server error");
}
