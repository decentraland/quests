use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use serde::Deserialize;
use std::{future::Future, pin::Pin};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct OptionalAuthUser {
    pub address: Option<String>,
}

impl FromRequest for OptionalAuthUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        let request = request.clone();
        Box::pin(async move {
            match super::verification(request.headers(), request.method().as_str(), request.path())
                .await
            {
                Ok(address) => Ok(OptionalAuthUser {
                    address: Some(address),
                }),
                // since it's optional auth, we do not return unathorization error
                Err(_) => Ok(OptionalAuthUser { address: None }),
            }
        })
    }
}
