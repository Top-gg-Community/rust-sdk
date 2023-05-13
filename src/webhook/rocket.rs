use crate::IncomingVote;
use rocket::{
  data::{self, Data, FromDataSimple},
  http::Status,
  request::Request,
  Outcome,
};

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
impl FromDataSimple for IncomingVote {
  type Error = ();

  fn from_data(request: &Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
    let headers = request.headers();

    if let Some(authorization) = headers.get_one("Authorization") {
      let mut body = Vec::with_capacity(
        headers
          .get_one("Content-Length")
          .and_then(|s| s.parse().ok())
          .unwrap_or_default(),
      );
      let _ = data.stream_to(&mut body);

      if let Ok(body) = String::from_utf8(body) {
        if let Ok(vote) = serde_json::from_str(&body) {
          return Outcome::Success(Self {
            authorization: authorization.to_owned(),
            vote,
          });
        }
      }
    }

    Outcome::Failure((Status::Unauthorized, ()))
  }
}
