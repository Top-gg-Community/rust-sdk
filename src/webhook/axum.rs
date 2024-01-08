use crate::VoteHandler;
use axum::{
  extract::State,
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Response},
  routing::post,
  Router,
};
use std::sync::Arc;

struct WebhookState<T> {
  state: Arc<T>,
  password: Arc<String>,
}

impl<T> Clone for WebhookState<T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      state: Arc::clone(&self.state),
      password: Arc::clone(&self.password),
    }
  }
}

async fn handler<T>(
  headers: HeaderMap,
  State(webhook): State<WebhookState<T>>,
  body: String,
) -> Response
where
  T: VoteHandler,
{
  if let Some(authorization) = headers.get("Authorization") {
    if let Ok(authorization) = authorization.to_str() {
      if authorization == *(webhook.password) {
        if let Ok(vote) = serde_json::from_str(&body) {
          webhook.state.voted(vote).await;

          return (StatusCode::OK, ()).into_response();
        }
      }
    }
  }

  (StatusCode::UNAUTHORIZED, ()).into_response()
}

/// Creates a new [`axum`] [`Router`] for adding an on-vote event handler to your application logic.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use axum::{routing::get, Router, Server};
/// use std::{net::SocketAddr, sync::Arc};
/// use topgg::{Vote, VoteHandler};
///
/// struct MyVoteHandler {}
///
/// #[axum::async_trait]
/// impl VoteHandler for MyVoteHandler {
///   async fn voted(&self, vote: Vote) {
///     println!("{:?}", vote);
///   }
/// }
///
/// async fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyVoteHandler {});
///
///   let app = Router::new().route("/", get(index)).nest(
///     "/webhook",
///     topgg::axum::webhook(env!("TOPGG_WEBHOOK_PASSWORD").to_string(), Arc::clone(&state)),
///   );
///
///   let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///
///   Server::bind(&addr)
///     .serve(app.into_make_service())
///     .await
///     .unwrap();
/// }
/// ```
#[inline(always)]
pub fn webhook<T>(password: String, state: Arc<T>) -> Router
where
  T: VoteHandler,
{
  Router::new()
    .route("/", post(handler::<T>))
    .with_state(WebhookState {
      state,
      password: Arc::new(password),
    })
}
