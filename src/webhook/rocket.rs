use crate::IncomingVote;
use rocket::{
  data::{Data, FromDataSimple, Outcome},
  http::Status,
  request::Request,
};

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
impl FromDataSimple for IncomingVote {
  type Error = ();

  fn from_data(request: &Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
    let headers = request.headers();

    if let Some(authorization) = headers.get_one("Authorization") {
      if let Ok(vote) = serde_json::from_reader(data.open()) {
        return Outcome::Success(Self {
          authorization: authorization.to_owned(),
          vote,
        });
      }
    }

    Outcome::Failure((Status::Unauthorized, ()))
  }
}
