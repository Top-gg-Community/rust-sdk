//! # Examples
//!
//! Basic usage:
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
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
//!   let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
//!   let state = MyVoteHandler {};
//!
//!   // POST /webhook
//!   let webhook = topgg::warp::webhook("webhook", password, state);
//!
//!   let routes = warp::get()
//!     .map(|| "Hello, World!")
//!     .or(webhook);
//!
//!   // this will always be a valid SocketAddr syntax,
//!   // therefore we can safely unwrap_unchecked this.
//!   let addr: SocketAddr = unsafe { "127.0.0.1:8080".parse().unwrap_unchecked() };
//!
//!   warp::serve(routes)
//!     .run(addr)
//!     .await
//! }
//! ```

use crate::{Vote, VoteHandler, WebhookState};
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, Filter, Rejection, Reply};

/// Creates a new `warp` [`Filter`] for adding an on-vote event handler to your application logic.
/// `state` here is your webhook handler.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use std::net::SocketAddr;
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
///   let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
///   let state = MyVoteHandler {};
///
///   // POST /webhook
///   let webhook = topgg::warp::webhook("webhook", password, state);
///
///   let routes = warp::get()
///     .map(|| "Hello, World!")
///     .or(webhook);
///
///   // this will always be a valid SocketAddr syntax,
///   // therefore we can safely unwrap_unchecked this.
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
  state: T,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone
where
  T: VoteHandler,
{
  let state = Arc::new(WebhookState { state, password });

  warp::post()
    .and(path(endpoint))
    .and(header("Authorization"))
    .and(body::json())
    .then(move |auth: String, vote: Vote| {
      let current_state = Arc::clone(&state);

      async move {
        if auth == current_state.password {
          current_state.state.voted(vote).await;

          StatusCode::OK
        } else {
          StatusCode::UNAUTHORIZED
        }
      }
    })
}
