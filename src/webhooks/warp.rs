use super::{Payload, PayloadResult};

use bytes::Bytes;
use chrono::Utc;
use warp::{Filter, Rejection, body, header, path};

/// Creates a new warp [`Filter`] for receiving webhook events.
///
/// # Example
///
/// ```rust,no_run
/// use topgg::PayloadResult;
/// use std::net::SocketAddr;
///
/// use warp::{Filter, http::StatusCode, reply};
///
/// #[tokio::main]
/// async fn main() {
///   // POST /webhook
///   let webhook =
///     topgg::warp::webhook("webhook", env!("TOPGG_WEBHOOK_SECRET").into()).then(|payload, _trace| async move {
///       match payload {
///         PayloadResult::Accepted(payload) => {
///           println!("{payload:?}");
///
///           reply::with_status("", StatusCode::NO_CONTENT)
///         }
///
///         PayloadResult::Forbidden => reply::with_status("Forbidden", StatusCode::FORBIDDEN),
///
///         PayloadResult::BadRequest => reply::with_status("Bad Request", StatusCode::BAD_REQUEST),
///
///         PayloadResult::Unauthorized => reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED),
///
///         PayloadResult::DeserializationFailure => reply::with_status("", StatusCode::NO_CONTENT),
///
///         PayloadResult::InternalServerError => {
///           reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
///         }
///       }
///     });
///
///   let routes = warp::get().map(|| "Hello, World!").or(webhook);
///
///   let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///
///   warp::serve(routes).run(addr).await
/// }
/// ```
#[must_use]
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
pub fn webhook(
  endpoint: &'static str,
  secret: String,
) -> impl Filter<Extract = (PayloadResult, String), Error = Rejection> + Clone {
  warp::post()
    .and(path(endpoint))
    .and(header("x-topgg-signature"))
    .and(body::content_length_limit(2 * 1024 * 1024))
    .and(body::bytes())
    .map(move |signature: String, body: Bytes| {
      let now = Utc::now();

      String::from_utf8(body.to_vec()).map_or(PayloadResult::BadRequest, |body| {
        Payload::new(now, body, &signature, &secret)
      })
    })
    .and(header("x-topgg-trace"))
}
