use super::Webhook;
use axum::{
  extract::State,
  http::{HeaderMap, StatusCode},
  response::IntoResponse,
  routing::post,
  Router,
};
use serde::de::DeserializeOwned;
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

/// Creates a new axum [`Router`] for receiving vote events.
///
/// # Example
///
/// ```rust,no_run
/// use axum::{routing::get, Router};
/// use topgg::{VoteEvent, Webhook};
/// use tokio::net::TcpListener;
/// use std::sync::Arc;
///
/// struct MyVoteListener {}
///
/// #[async_trait::async_trait]
/// impl Webhook<VoteEvent> for MyVoteListener {
///   async fn callback(&self, vote: VoteEvent) {
///     println!("A user with the ID of {} has voted us on Top.gg!", vote.voter_id);
///   }
/// }
///
/// async fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyVoteListener {});
///
///   let router = Router::new().route("/", get(index)).nest(
///     "/votes",
///     topgg::axum::webhook(env!("MY_TOPGG_WEBHOOK_SECRET").to_string(), Arc::clone(&state)),
///   );
///
///   let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
///
///   axum::serve(listener, router).await.unwrap();
/// }
/// ```
#[inline(always)]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
pub fn webhook<D, T>(password: String, state: Arc<T>) -> Router
where
  D: DeserializeOwned + Send,
  T: Webhook<D>,
{
  Router::new()
    .route(
      "/",
      post(
        async |headers: HeaderMap, State(webhook): State<WebhookState<T>>, body: String| {
          if let Some(authorization) = headers.get("Authorization") {
            if let Ok(authorization) = authorization.to_str() {
              if authorization == *(webhook.password) {
                if let Ok(data) = serde_json::from_str(&body) {
                  webhook.state.callback(data).await;

                  return (StatusCode::NO_CONTENT, ()).into_response();
                }
              }
            }
          }

          (StatusCode::UNAUTHORIZED, ()).into_response()
        },
      ),
    )
    .with_state(WebhookState {
      state,
      password: Arc::new(password),
    })
}
