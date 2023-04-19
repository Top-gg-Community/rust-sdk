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

use crate::{Vote, VoteHandler};
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, post, Filter};

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
pub fn webhook<T>(endpoint: &'static str, password: String, state: T) -> impl Filter
where
  T: VoteHandler,
{
  let state = Arc::new(state);
  let password = Arc::new(password);

  path(endpoint)
    .and(header::<String>("Authorization"))
    .and(body::json())
    .then(move |auth, vote: Vote| {
      let current_state = Arc::clone(&state);
      let current_password = Arc::clone(&password);

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
