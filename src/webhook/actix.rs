use crate::{IncomingVote, Vote};
use actix_web::{
  dev::Payload,
  error::{Error, ErrorUnauthorized},
  web::Json,
  FromRequest, HttpRequest,
};
use core::{future::Future, pin::Pin};

#[cfg_attr(docsrs, doc(cfg(feature = "actix")))]
impl FromRequest for IncomingVote {
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    let json: Vote = Json::from_request(req, payload);
    let req = req.clone();

    Box::pin(async move {
      let headers = req.headers();

      if let Some(authorization) = headers.get("Authorization") {
        if let Ok(authorization) = authorization.to_str() {
          return Ok(Self {
            authorization: authorization.to_owned(),
            vote: json.await?.into_inner(),
          });
        }
      }

      Err(ErrorUnauthorized("401"))
    })
  }
}
