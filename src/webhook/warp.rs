//! # Examples
//!
//! Basic usage:
//!
//! ```rust,no_run
//! struct MyVoteHandler {}
//!
//! #[async_trait::async_trait]
//! impl topgg::VoteHandler for MyVoteHandler {
//!   async fn voted(&self, vote: topgg::Vote) {
//!     // your application logic here
//!   }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!   let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
//!   let state = MyVoteHandler {};
//!   
//!   // POST /dblwebhook
//!   let webhook = topgg::warp::webhook("dblwebhook", password, state);   
//!   let routes = warp::post().and(webhook);
//!
//!   warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
//! }
//! ```

use crate::{Vote, VoteHandler, WebhookState};
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, Filter};

/// Creates a new `warp` [`Filter`] for adding an on-vote event handler to your application logic.
/// `state` here is your webhook handler.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// struct MyVoteHandler {}
///
/// #[async_trait::async_trait]
/// impl topgg::VoteHandler for MyVoteHandler {
///   async fn voted(&self, vote: topgg::Vote) {
///     // your application logic here
///   }
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
///   let state = MyVoteHandler {};
///   
///   // POST /dblwebhook
///   let webhook = topgg::warp::webhook("dblwebhook", password, state);   
///   let routes = warp::post().and(webhook);
///
///   warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
pub fn webhook<T>(endpoint: &'static str, password: String, state: T) -> impl Filter
where
  T: VoteHandler,
{
  let state = Arc::new(WebhookState { state, password });

  path(endpoint)
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
