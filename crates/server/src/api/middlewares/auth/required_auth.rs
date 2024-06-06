use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use serde::Deserialize;
use std::{future::Future, pin::Pin};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct RequiredAuthUser {
    pub address: String,
}

impl FromRequest for RequiredAuthUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let request = request.clone();
        Box::pin(async move {
            super::verification(request.headers(), request.method().as_str(), request.path())
                .await
                .map(|address| RequiredAuthUser { address })
                .map_err(|_| ErrorUnauthorized("Unathorized"))
        })
    }
}
