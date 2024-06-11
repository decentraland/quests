use actix_web::http::header::HeaderMap;
use dcl_crypto_middleware_rs::signed_fetch::{verify, AuthMiddlewareError, VerificationOptions};
use std::collections::HashMap;

pub mod optional_auth;
pub mod required_auth;

async fn verification(
    headers: &HeaderMap,
    method: &str,
    path: &str,
) -> Result<String, AuthMiddlewareError> {
    let headers = headers
        .iter()
        .map(|(key, val)| (key.to_string(), val.to_str().unwrap_or("").to_string()))
        .collect::<HashMap<String, String>>();

    verify(method, path, headers, VerificationOptions::default())
        .await
        .map(|address| address.to_string().to_ascii_lowercase())
}
