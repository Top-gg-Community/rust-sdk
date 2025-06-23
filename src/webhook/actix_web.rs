use crate::Incoming;
use actix_web::{
  dev::Payload,
  error::{Error, ErrorUnauthorized},
  web::Json,
  FromRequest, HttpRequest,
};
use std::{
  future::Future,
  pin::Pin,
  task::{ready, Context, Poll},
};
use serde::de::DeserializeOwned;

#[doc(hidden)]
pub struct IncomingFut<T: DeserializeOwned> {
  req: HttpRequest,
  json_fut: <Json<T> as FromRequest>::Future,
}

impl<T> Future for IncomingFut<T>
where
  T: DeserializeOwned,
{
  type Output = Result<Incoming<T>, Error>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if let Ok(json) = ready!(Pin::new(&mut self.json_fut).poll(cx)) {
      let headers = self.req.headers();

      if let Some(authorization) = headers.get("Authorization") {
        if let Ok(authorization) = authorization.to_str() {
          return Poll::Ready(Ok(Incoming {
            authorization: authorization.to_owned(),
            data: json.into_inner(),
          }));
        }
      }
    }

    Poll::Ready(Err(ErrorUnauthorized("401")))
  }
}

#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl<T> FromRequest for Incoming<T>
where
  T: DeserializeOwned,
{
  type Error = Error;
  type Future = IncomingFut<T>;

  #[inline(always)]
  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    IncomingFut {
      req: req.clone(),
      json_fut: Json::from_request(req, payload),
    }
  }
}
