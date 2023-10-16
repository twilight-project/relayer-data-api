use diesel::{prelude::PgConnection, Connection};
use http::{Request, StatusCode};
use hyper::{body::to_bytes, server::Server, Body, Response};
use log::debug;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::{make::Shared, ServiceBuilder};
use twilight_relayerAPI::{auth::AuthInfo, database::AddressCustomerId};
use verify_keplr_sign::{verify_arbitrary, Signature};

#[derive(Deserialize)]
pub struct Auth {
    pub account_address: String,
    pub signature: Signature,
    pub data: String,
}

enum VerifyResult {
    Valid(String),
    InvalidJson,
    Unauthorized,
}

async fn verify_signature(request: Request<Body>) -> VerifyResult {
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
        Err(e) => return VerifyResult::InvalidJson,
    };

    let is_ok = verify_arbitrary(
        &account_address,
        &signature.pub_key.sig_value,
        data.as_bytes(),
        &signature,
    );

    if !is_ok {
        return VerifyResult::Unauthorized;
    }

    VerifyResult::Valid(account_address)
}

async fn login_handler(account_address: String) -> Result<Response<Body>, http::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("No database url set!");
    let mut conn = match PgConnection::establish(&database_url) {
        Ok(c) => c,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal db error".into());
        }
    };

    let customer_id = match AddressCustomerId::get_or_create(&mut conn, &account_address) {
        Ok(customer) => customer.customer_id,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal db error".into());
        }
    };

    let token = AuthInfo {
        account_address,
        customer_id,
    };
    let response = serde_json::to_string(&token).expect("Could not serialize");

    return Response::builder()
        .status(StatusCode::OK)
        .body(response.into());
}

async fn register_handler(address: String) -> Result<Response<Body>, http::Error> {
    return Response::builder().status(StatusCode::OK).body("OK".into());
}

async fn handler(request: Request<Body>) -> Result<Response<Body>, http::Error> {
    debug!("Auth request");
    let uri = request.uri().to_string();

    if &uri != "/login" && &uri != "/register" {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty());
    }

    let address = match verify_signature(request).await {
        VerifyResult::Valid(address) => address,
        VerifyResult::InvalidJson => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty());
        }
        VerifyResult::Unauthorized => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty());
        }
    };

    if &uri == "/login" {
        login_handler(address).await
    } else {
        register_handler(address).await
    }
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
