use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use actix_web_lab::middleware::{from_fn, Next};
use dcl_crypto::Address;
use dcl_crypto_middleware_rs::signed_fetch::{verify, VerificationOptions};
use std::collections::HashMap;

// This middlware is intended for routes where the auth is REQUIRED
pub fn dcl_auth_middleware<S, B>(
    protected_routes: [&'static str; 4],
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
        verification(
            req,
            &protected_routes,
            |req, verification_result| async move {
                match verification_result {
                    VerificationResult::VerificationPassed(address) => {
                        {
                            let mut extensions = req.extensions_mut();
                            extensions.insert(Some(address));
                        }
                        next.call(req).await
                    }
                    VerificationResult::VerificationFailed => Err(ErrorUnauthorized("Unathorized")),
                    VerificationResult::NotProtectedRoute => next.call(req).await,
                }
            },
        )
        .await
    })
}

// This middlware is intended for routes where the auth is optional
pub fn dcl_optional_auth_middleware<S, B>(
    protected_routes: [&'static str; 1],
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
        verification(
            req,
            &protected_routes,
            |req, verification_result| async move {
                match verification_result {
                    VerificationResult::VerificationPassed(address) => {
                        {
                            let mut extensions = req.extensions_mut();
                            extensions.insert(Some(address));
                        }
                        next.call(req).await
                    }
                    VerificationResult::VerificationFailed => {
                        {
                            let mut extensions = req.extensions_mut();
                            extensions.insert(Option::<Address>::None);
                        }
                        next.call(req).await
                    }
                    VerificationResult::NotProtectedRoute => next.call(req).await,
                }
            },
        )
        .await
    })
}

async fn verification<
    B: MessageBody + 'static,
    F: core::future::Future<Output = Result<ServiceResponse<B>, Error>>,
>(
    req: ServiceRequest,
    protected_routes: &[&str],
    cb: impl FnOnce(ServiceRequest, VerificationResult) -> F,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let path = if let Some(path) = req.match_pattern() {
        path.to_string()
    } else {
        req.path().to_string()
    };

    let route = format!("{}:{}", req.method(), path);

    if protected_routes.contains(&route.as_str()) {
        let headers = req
            .headers()
            .iter()
            .map(|(key, val)| (key.to_string(), val.to_str().unwrap_or("").to_string()))
            .collect::<HashMap<String, String>>();

        if let Ok(address) = verify(
            req.method().as_str(),
            req.path(),
            headers,
            VerificationOptions::default(),
        )
        .await
        {
            cb(req, VerificationResult::VerificationPassed(address)).await
        } else {
            cb(req, VerificationResult::VerificationFailed).await
        }
    } else {
        cb(req, VerificationResult::NotProtectedRoute).await
    }
}

enum VerificationResult {
    VerificationFailed,
    VerificationPassed(Address),
    NotProtectedRoute,
}
