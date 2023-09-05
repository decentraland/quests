use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::HeaderMap,
    Error, HttpMessage,
};
use actix_web_lab::middleware::{from_fn, Next};
use dcl_crypto::Address;
use dcl_crypto_middleware_rs::signed_fetch::{verify, AuthMiddlewareError, VerificationOptions};
use std::collections::HashMap;

// This middlware is intended for routes where the auth is REQUIRED
pub fn dcl_auth_middleware<S, B>(
    required_auth_routes: [&'static str; 5],
    optional_auth_routes: [&'static str; 3],
) -> impl Transform<
    S,
    ServiceRequest,
    Response = ServiceResponse<impl MessageBody>,
    Error = Error,
    InitError = (),
>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    from_fn(move |req: ServiceRequest, next: Next<B>| async move {
        let path = if let Some(path) = req.match_pattern() {
            path.to_string()
        } else {
            req.path().to_string()
        };

        let route = format!("{}:{}", req.method(), path);

        if required_auth_routes.contains(&route.as_str()) {
            if let Ok(address) =
                verification(req.headers(), req.method().as_str(), req.path()).await
            {
                {
                    let mut extensions = req.extensions_mut();
                    extensions.insert(address);
                }
                next.call(req).await
            } else {
                Err(ErrorUnauthorized("Unathorized"))
            }
        } else if optional_auth_routes.contains(&route.as_str()) {
            match verification(req.headers(), req.method().as_str(), req.path()).await {
                Ok(address) => {
                    {
                        let mut extensions = req.extensions_mut();
                        extensions.insert(address);
                    }
                    next.call(req).await
                }
                Err(_) => next.call(req).await,
            }
        } else {
            next.call(req).await
        }
    })
}

async fn verification(
    headers: &HeaderMap,
    method: &str,
    path: &str,
) -> Result<Address, AuthMiddlewareError> {
    let headers = headers
        .iter()
        .map(|(key, val)| (key.to_string(), val.to_str().unwrap_or("").to_string()))
        .collect::<HashMap<String, String>>();

    verify(method, path, headers, VerificationOptions::default()).await
}
