use super::IncomingPayload;
use std::{
  fmt::{self, Display, Formatter},
  future::Future,
  pin::Pin,
  task::{Context, Poll, ready},
  time::{Duration, Instant},
};

use actix_web::{
  FromRequest, HttpRequest, HttpResponse, ResponseError, body::BoxBody, dev::Payload,
  http::StatusCode,
};
use chrono::{DateTime, Utc};
use futures_core::stream::Stream;

#[doc(hidden)]
#[derive(Debug)]
pub enum IncomingPayloadError {
  BadRequest,
  Timeout,
}

impl Display for IncomingPayloadError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::BadRequest => "Bad Request.",

      Self::Timeout => "Request timed out.",
    })
  }
}

impl ResponseError for IncomingPayloadError {
  fn error_response(&self) -> HttpResponse<BoxBody> {
    match self {
      Self::BadRequest => HttpResponse::BadRequest().body("Bad Request"),

      Self::Timeout => HttpResponse::RequestTimeout().body("Request timed out"),
    }
  }

  fn status_code(&self) -> StatusCode {
    match self {
      Self::BadRequest => StatusCode::BAD_REQUEST,

      Self::Timeout => StatusCode::REQUEST_TIMEOUT,
    }
  }
}

#[doc(hidden)]
pub struct IncomingPayloadFut {
  req: HttpRequest,
  payload: Payload,
  body: Vec<u8>,
  start: Instant,
  now: DateTime<Utc>,
}

impl IncomingPayloadFut {
  fn timed_out(&self) -> bool {
    self.start.elapsed() > Duration::from_secs(5)
  }
}

impl Future for IncomingPayloadFut {
  type Output = Result<IncomingPayload, IncomingPayloadError>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.timed_out() {
      return Poll::Ready(Err(IncomingPayloadError::Timeout));
    }

    while let Some(body) = ready!(Pin::new(&mut self.payload).poll_next(cx)) {
      if self.timed_out() {
        return Poll::Ready(Err(IncomingPayloadError::Timeout));
      }

      if let Ok(body) = body {
        self.body.extend_from_slice(&body);
      } else {
        return Poll::Ready(Err(IncomingPayloadError::BadRequest));
      }
    }

    let headers = self.req.headers();

    if let (Some(signature), Some(trace)) = (
      headers.get("x-topgg-signature"),
      headers.get("x-topgg-trace"),
    ) && let (Ok(signature), Ok(trace), Ok(body)) = (
      signature.to_str(),
      trace.to_str(),
      str::from_utf8(&self.body),
    ) && let Some(incoming) = IncomingPayload::new(self.now, body.into(), signature, trace)
    {
      return Poll::Ready(Ok(incoming));
    }

    Poll::Ready(Err(IncomingPayloadError::BadRequest))
  }
}

#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl FromRequest for IncomingPayload {
  type Error = IncomingPayloadError;
  type Future = IncomingPayloadFut;

  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    IncomingPayloadFut {
      req: req.clone(),
      payload: payload.take(),
      body: vec![],
      start: Instant::now(),
      now: Utc::now(),
    }
  }
}
