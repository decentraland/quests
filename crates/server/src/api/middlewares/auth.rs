use std::collections::HashMap;

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use actix_web_lab::middleware::{from_fn, Next};
use dcl_crypto_middleware_rs::signed_fetch::{verify, VerificationOptions};

pub fn dcl_auth_middlware<S, B>() -> impl Transform<
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
            {
                let mut extensions = req.extensions_mut();
                extensions.insert(address);
            }

            next.call(req).await
        } else {
            Err(ErrorUnauthorized("Unathorized"))
        }
    })
}
