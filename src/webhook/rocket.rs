use crate::Incoming;
use rocket::{
  data::{Data, FromData, Outcome},
  http::Status,
  request::Request,
  serde::json::Json,
};
use serde::de::DeserializeOwned;

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
#[rocket::async_trait]
impl<'r, T> FromData<'r> for Incoming<T>
where
  T: DeserializeOwned,
{
  type Error = ();

  async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
    let headers = request.headers();

    if let Some(authorization) = headers.get_one("Authorization") {
      if let Outcome::Success(data) = <Json<T> as FromData>::from_data(request, data).await {
        return Outcome::Success(Self {
          authorization: authorization.to_owned(),
          data: data.into_inner(),
        });
      }
    }

    Outcome::Error((Status::Unauthorized, ()))
  }
}
