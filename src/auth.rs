use chrono::prelude::*;
use chrono::Days;
use futures_util::future::BoxFuture;
use hmac::{Hmac, Mac};
use http::{header::AUTHORIZATION, Request, Response, StatusCode};
use hyper::Body;
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use log::error;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tower_http::auth::AsyncAuthorizeRequest;

const TOKEN_EXPIRY_DAYS: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthToken {
    userid: String,
    is_admin: bool,
    // TODO: parse as DateTime<Utc>
    exp: usize,
}

impl AuthToken {
    // TODO: perms on this.
    pub fn new(userid: String) -> AuthToken {
        let expiry_days = Days::new(TOKEN_EXPIRY_DAYS);
        let exp = Utc::now()
            .checked_add_days(expiry_days)
            .unwrap()
            .timestamp() as usize;
        AuthToken {
            userid,
            is_admin: false,
            exp,
        }
    }

    pub fn signed_token(self) -> Result<String, jwt::error::Error> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(b"test_secret").expect("Bad key");
        self.sign_with_key(&key)
    }
}

pub struct UserId(String);

#[derive(Clone, Copy)]
pub struct TwilightAuth;

impl<B> AsyncAuthorizeRequest<B> for TwilightAuth
where
    B: Send + Sync + 'static,
{
    type RequestBody = B;
    type ResponseBody = Body;
    type Future = BoxFuture<'static, Result<Request<B>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<B>) -> Self::Future {
        Box::pin(async {
            // TODO: load from env var
            let key: Hmac<Sha256> = Hmac::new_from_slice(b"test_secret").expect("Bad key");
            if let Some(user_id) = check_auth(&request, &key).await {
                request.extensions_mut().insert(user_id);

                Ok(request)
            } else {
                let unauthorized_response = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap();

                Err(unauthorized_response)
            }
        })
    }
}

pub async fn check_auth<B>(request: &Request<B>, key: &Hmac<Sha256>) -> Option<UserId>
where
    B: Send + Sync + 'static,
{
    match request.headers().get(AUTHORIZATION) {
        Some(header) => match header.to_str() {
            Ok(strval) => {
                let mut split = strval.split(' ');
                let bearer = split.next();
                let token = split.next();

                if bearer != Some("Bearer") {
                    return None;
                }

                if token.is_none() {
                    return None;
                }

                match token.unwrap().verify_with_key(key) {
                    Ok::<Token<Header, AuthToken, _>, jwt::Error>(token) => {
                        let _ = token.header();
                        let claims = token.claims();
                        Some(UserId(claims.userid.clone()))
                    }
                    Err(e) => {
                        error!("Auth error {:?}", e);
                        None
                    }
                }
            }
            e => {
                error!("Auth error {:?}", e);
                None
            }
        },
        None => None,
    }
}
