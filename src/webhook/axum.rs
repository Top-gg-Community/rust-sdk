//! # Examples
//!
//! Basic usage:
//!
//! ```rust,no_run
//! use axum::{Router, Server};
//! use std::net::SocketAddr;
//!
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
//!   let app = Router::new()
//!     .nest("/dblwebhook", topgg::axum::webhook(password, state));
//!   
//!   let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!
//!   Server::bind(&addr)
//!     .serve(app.into_make_service())
//!     .await
//!     .unwrap();
//! }
//! ```

use crate::{VoteHandler, WebhookState};
use axum::{
  extract::State,
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Response},
  routing::post,
  Router,
};
use std::sync::Arc;

async fn handler<T>(
  headers: HeaderMap,
  State(webhook): State<Arc<WebhookState<T>>>,
  body: String,
) -> Response
where
  T: VoteHandler,
{
  if let Some(authorization) = headers.get("Authorization") {
    if let Ok(authorization) = authorization.to_str() {
      if authorization == webhook.password {
        if let Ok(vote) = serde_json::from_str(&body) {
          webhook.state.voted(vote).await;

          return (StatusCode::OK, ()).into_response();
        }
      }
    }
  }

  (StatusCode::UNAUTHORIZED, ()).into_response()
}

/// Creates a new `axum` [`Router`] for adding an on-vote event handler to your application logic.
/// `state` here is your webhook handler.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use axum::{Router, Server};
/// use std::net::SocketAddr;
///
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
///   let app = Router::new()
///     .nest("/dblwebhook", topgg::axum::webhook(password, state));
///   
///   let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
///
///   Server::bind(&addr)
///     .serve(app.into_make_service())
///     .await
///     .unwrap();
/// }
/// ```
pub fn webhook<T>(password: String, state: T) -> Router
where
  T: VoteHandler,
{
  Router::new()
    .route("/", post(handler::<T>))
    .with_state(Arc::new(WebhookState { state, password }))
}
