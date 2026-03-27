use super::IncomingPayload;
use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll, ready},
};

use actix_web::{
  FromRequest, HttpRequest,
  dev::Payload,
  error::{Error, ErrorBadRequest, ErrorUnauthorized},
};
use futures_core::stream::Stream;

#[doc(hidden)]
pub struct IncomingPayloadFut {
  req: HttpRequest,
  payload: Payload,
  body: Vec<u8>,
}

impl Future for IncomingPayloadFut {
  type Output = Result<IncomingPayload, Error>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    while let Some(body) = ready!(Pin::new(&mut self.payload).poll_next(cx)) {
      match body {
        Ok(body) => self.body.extend_from_slice(&body),

        Err(_) => return Poll::Ready(Err(ErrorBadRequest("400"))),
      }
    }

    let headers = self.req.headers();

    if let (Some(signature), Some(trace)) = (
      headers.get("x-topgg-signature"),
      headers.get("x-topgg-trace"),
    ) && let (Ok(signature), Ok(trace)) = (signature.to_str(), trace.to_str())
      && let Some(incoming) = IncomingPayload::new(signature, self.body.clone(), trace)
    {
      return Poll::Ready(Ok(incoming));
    }

    Poll::Ready(Err(ErrorUnauthorized("401")))
  }
}

#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl FromRequest for IncomingPayload {
  type Error = Error;
  type Future = IncomingPayloadFut;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    IncomingPayloadFut {
      req: req.clone(),
      payload: payload.take(),
      body: vec![],
    }
  }
}
