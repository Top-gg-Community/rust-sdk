use crate::{IncomingVote, Vote};
use rocket::{
  data::{Data, FromData, Outcome},
  http::Status,
  request::Request,
  serde::json::Json,
};

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
#[rocket::async_trait]
impl<'r> FromData<'r> for IncomingVote {
  type Error = ();

  async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
    let headers = request.headers();

    if let Some(authorization) = headers.get_one("Authorization") {
      if let Outcome::Success(vote) = <Json<Vote> as FromData>::from_data(request, data).await {
        return Outcome::Success(Self {
          authorization: authorization.to_owned(),
          vote: vote.into_inner(),
        });
      }
    }

    Outcome::Error((Status::Unauthorized, ()))
  }
}
