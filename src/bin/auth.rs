use http::{Request, StatusCode};
use hyper::{body::to_bytes, server::Server, Body, Response};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::{make::Shared, ServiceBuilder};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use twilight_relayerAPI::auth::AuthToken;
use verify_keplr_sign::{verify_arbitrary, Signature};

#[derive(Deserialize)]
pub struct Auth {
    pub account_address: String,
    pub signature: Signature,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub auth_token: String,
}

async fn handler(request: Request<Body>) -> Result<Response<Body>, http::Error> {
    if request.uri() != "/auth" {
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

    let auth_token = AuthToken::new(account_address);

    match auth_token.signed_token() {
        Ok(auth_token) => {
            let token = AuthResponse { auth_token };
            let response = serde_json::to_string(&token).expect("Could not serialize");

            return Response::builder()
                .status(StatusCode::OK)
                .body(response.into());
        }
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty());
        }
    }
}

#[tokio::main]
async fn main() {
    let service = ServiceBuilder::new().service_fn(handler);

    // And run our service using `hyper`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3112));
    Server::bind(&addr)
        .serve(Shared::new(service))
        .await
        .expect("server error");
}
