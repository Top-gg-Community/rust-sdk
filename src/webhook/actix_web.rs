use crate::{IncomingVote, Vote};
use actix_web::{
  dev::Payload,
  error::{Error, ErrorUnauthorized},
  web::Json,
  FromRequest, HttpRequest,
};
use core::{
  future::Future,
  pin::Pin,
  task::{ready, Context, Poll},
};

#[doc(hidden)]
pub struct IncomingVoteFut {
  req: HttpRequest,
  json_fut: <Json<Vote> as FromRequest>::Future,
}

impl Future for IncomingVoteFut {
  type Output = Result<IncomingVote, Error>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if let Ok(json) = ready!(Pin::new(&mut self.json_fut).poll(cx)) {
      let headers = self.req.headers();

      if let Some(authorization) = headers.get("Authorization") {
        if let Ok(authorization) = authorization.to_str() {
          return Poll::Ready(Ok(IncomingVote {
            authorization: authorization.to_owned(),
            vote: json.into_inner(),
          }));
        }
      }
    }

    Poll::Ready(Err(ErrorUnauthorized("401")))
  }
}

#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl FromRequest for IncomingVote {
  type Error = Error;
  type Future = IncomingVoteFut;

  #[inline(always)]
  fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
    IncomingVoteFut {
      req: req.clone(),
      json_fut: Json::from_request(req, payload),
    }
  }
}
