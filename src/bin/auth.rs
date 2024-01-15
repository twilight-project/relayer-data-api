use diesel::{prelude::PgConnection, Connection};
use digest::{CtOutput, Output, OutputSizeUser};
use hmac::{Hmac, Mac};
use http::{Request, StatusCode};
use hyper::{body::to_bytes, server::Server, Body, Response};
use log::debug;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::net::SocketAddr;
use tower::{make::Shared, ServiceBuilder};
use twilight_relayerAPI::{
    auth::{AuthInfo, UserInfo},
    database::{AddressCustomerId, CustomerApiKeyLinking},
};
use verify_keplr_sign::{verify_arbitrary, Signature};

type HS = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct SigCheck {
    pub api_key: String,
    pub sig: String,
    pub datetime: String,
    pub body: String,
}

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

    let (api_key, api_secret) = match CustomerApiKeyLinking::get_or_create(&mut conn, customer_id) {
        Ok(link) => (link.api_key, link.api_salt_key),
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal db error".into());
        }
    };

    let token = AuthInfo {
        api_key,
        api_secret,
    };
    let response = serde_json::to_string(&token).expect("Could not serialize");

    return Response::builder()
        .status(StatusCode::OK)
        .body(response.into());
}

async fn check_signature(request: Request<Body>) -> Result<Response<Body>, http::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("No database url set!");
    let mut conn = match PgConnection::establish(&database_url) {
        Ok(c) => c,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal db error".into());
        }
    };

    let request: Vec<u8> = to_bytes(request.into_body())
        .await
        .expect("Bad bytes")
        .to_vec();
    let SigCheck {
        api_key,
        sig,
        body,
        datetime,
    } = serde_json::from_slice(&request).expect("f");

    let key = match CustomerApiKeyLinking::get_key(&mut conn, api_key) {
        Ok(k) => k,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("No customer with that key".into());
        }
    };

    let Ok(received) = hex::decode(&sig) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Bad request".into());
    };

    if received.len() != HS::output_size() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty());
    }
    let output = Output::<HS>::clone_from_slice(&received);
    let digest: CtOutput<HS> = CtOutput::from(&output);

    let Ok(mut mac) = HS::new_from_slice(key.api_salt_key.as_bytes()) else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Hasher error".into());
    };
    mac.update(body.as_bytes());
    let calced = mac.finalize();

    if calced != digest {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Invalid digest".into());
    }

    let response = UserInfo {
        customer_id: key.customer_account_id,
    };

    let Ok(body) = serde_json::to_string(&response) else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("JSON serde error".into());
    };

    return Response::builder().status(StatusCode::OK).body(body.into());
}

async fn register_handler(address: String) -> Result<Response<Body>, http::Error> {
    return Response::builder().status(StatusCode::OK).body("OK".into());
}

async fn handler(request: Request<Body>) -> Result<Response<Body>, http::Error> {
    debug!("Auth request");
    let uri = request.uri().to_string();

    if &uri == "/check" {
        return check_signature(request).await;
    }

    if &uri != "/login" && &uri != "/register" {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".into());
    }

    let address = match verify_signature(request).await {
        VerifyResult::Valid(address) => address,
        VerifyResult::InvalidJson => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid signature".into());
        }
        VerifyResult::Unauthorized => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized".into());
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
