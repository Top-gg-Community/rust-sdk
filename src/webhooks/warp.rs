use super::Payload;

use bytes::Bytes;
use warp::{Filter, Rejection, body, header, path};

/// Creates a new warp [`Filter`] for receiving webhook events.
///
/// # Example
///
/// ```rust,no_run
/// use std::net::SocketAddr;
///
/// use warp::{http::StatusCode, reply, Filter};
///
/// #[tokio::main]
/// async fn main() {
///   // POST /webhook
///   let webhook = topgg::warp::webhook(
///     "webhook",
///     env!("TOPGG_WEBHOOK_SECRET").to_string()
///   ).then(|payload, _trace| async move {
///     match payload {
///       Some(payload) => {
///         println!("{payload:?}");
///
///         reply::with_status("", StatusCode::NO_CONTENT)
///       },
///
///       None => reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED)
///     }
///   });
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
) -> impl Filter<Extract = (Option<Payload>, String), Error = Rejection> + Clone {
  warp::post()
    .and(path(endpoint))
    .and(header("x-topgg-signature"))
    .and(body::bytes())
    .map(move |signature: String, body: Bytes| {
      str::from_utf8(&body)
        .ok()
        .and_then(|body| Payload::new(&signature, body, &secret))
    })
    .and(header("x-topgg-trace"))
}
