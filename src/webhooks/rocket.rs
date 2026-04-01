use super::IncomingPayload;
use std::time::Duration;

use chrono::Utc;
use rocket::{
  data::{Data, FromData, Outcome, ToByteUnit},
  http::Status,
  request::Request,
};
use tokio::time::timeout;

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
#[rocket::async_trait]
impl<'r> FromData<'r> for IncomingPayload {
  type Error = ();

  async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
    let now = Utc::now();
    let headers = request.headers();

    if let (Some(signature), Some(trace)) = (
      headers.get_one("x-topgg-signature"),
      headers.get_one("x-topgg-trace"),
    ) {
      match timeout(
        Duration::from_secs(5),
        data.open(2.mebibytes()).into_bytes(),
      )
      .await
      {
        Ok(Ok(body)) => {
          if let Ok(body) = String::from_utf8(body.into_inner())
            && let Some(payload) = Self::new(now, body, signature, trace)
          {
            return Outcome::Success(payload);
          }
        }

        Err(_) => return Outcome::Error((Status::RequestTimeout, ())),

        _ => {}
      }
    }

    Outcome::Error((Status::BadRequest, ()))
  }
}
