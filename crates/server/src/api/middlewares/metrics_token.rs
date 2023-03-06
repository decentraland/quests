use std::{collections::HashMap};

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web, Error,
};
use actix_web_lab::middleware::{from_fn, Next};

fn validate_token(
    bearer_token: String,
    query: web::Query<HashMap<String, String>>,
    request: &ServiceRequest,
) -> Result<(), Error> {
    let path = request.path();
    if path == "/metrics" {
        let token = query.get("bearer_token");
        if bearer_token.is_empty() {
            log::error!("missing wkc_metrics_bearer_token in configuration");
            return Err(ErrorInternalServerError(""));
        }
        if let Some(token) = token {
            if token != &bearer_token {
                return Err(ErrorUnauthorized("Invalid token"));
            }
        }
    }

    Ok(())
}

pub fn metrics_token<S, B>(
    bearer_token: &str,
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
    let token = bearer_token.to_owned();
    from_fn(move |query, req, next: Next<B>| {
        let token = token.clone();
        async move {
            validate_token(token, query, &req)?;
            next.call(req).await
        }
    })
}
