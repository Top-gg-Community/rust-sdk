//! # Examples
//!
//! Basic usage:
//!
//! ```rust,no_run
//! use std::{net::SocketAddr, sync::Arc};
//! use topgg::{Vote, VoteHandler};
//! use warp::Filter;
//!
//! struct MyVoteHandler {}
//!
//! #[async_trait::async_trait]
//! impl VoteHandler for MyVoteHandler {
//!   async fn voted(&self, vote: Vote) {
//!     println!("{:?}", vote);
//!   }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!   let state = Arc::new(MyVoteHandler {});
//!
//!   // POST /webhook
//!   let webhook = topgg::warp::webhook("webhook", env!("TOPGG_WEBHOOK_PASSWORD").to_string(), state.clone());
//!
//!   let routes = warp::get()
//!     .map(|| "Hello, World!")
//!     .or(webhook);
//!
//!   // SAFETY: this will always be a valid SocketAddr syntax.
//!   let addr: SocketAddr = unsafe { "127.0.0.1:8080".parse().unwrap_unchecked() };
//!
//!   warp::serve(routes)
//!     .run(addr)
//!     .await
//! }
//! ```

use crate::{Vote, VoteHandler};
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, Filter, Rejection, Reply};

/// Creates a new `warp` [`Filter`] for adding an on-vote event handler to your application logic.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use std::{net::SocketAddr, sync::Arc};
/// use topgg::{Vote, VoteHandler};
/// use warp::Filter;
///
/// struct MyVoteHandler {}
///
/// #[async_trait::async_trait]
/// impl VoteHandler for MyVoteHandler {
///   async fn voted(&self, vote: Vote) {
///     println!("{:?}", vote);
///   }
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyVoteHandler {});
///
///   // POST /webhook
///   let webhook = topgg::warp::webhook("webhook", env!("TOPGG_WEBHOOK_PASSWORD").to_string(), state.clone());
///
///   let routes = warp::get()
///     .map(|| "Hello, World!")
///     .or(webhook);
///
///   // SAFETY: this will always be a valid SocketAddr syntax.
///   let addr: SocketAddr = unsafe { "127.0.0.1:8080".parse().unwrap_unchecked() };
///
///   warp::serve(routes)
///     .run(addr)
///     .await
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
pub fn webhook<T>(
  endpoint: &'static str,
  password: String,
  state: Arc<T>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone
where
  T: VoteHandler,
{
  let password = Arc::new(password);

  warp::post()
    .and(path(endpoint))
    .and(header("Authorization"))
    .and(body::json())
    .then(move |auth: String, vote: Vote| {
      let current_state = state.clone();
      let current_password = password.clone();

      async move {
        if auth == *current_password {
          current_state.voted(vote).await;

          StatusCode::OK
        } else {
          StatusCode::UNAUTHORIZED
        }
      }
    })
}
