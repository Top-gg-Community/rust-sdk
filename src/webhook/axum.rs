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

          return (StatusCode::NO_CONTENT, ()).into_response();
        }
      }
    }
  }

  (StatusCode::UNAUTHORIZED, ()).into_response()
}

/// Creates a new axum [`Router`] for receiving vote events.
///
/// # Example
///
/// Basic usage:
///
/// ```rust
/// use axum::{routing::get, Router};
/// use std::sync::Arc;
/// use tokio::net::TcpListener;
/// use topgg::{Vote, VoteHandler};
/// #
/// # use std::time::Duration;
/// # use tokio::{sync::{oneshot, Notify}, time::sleep};
/// #
/// # async fn test_request() -> reqwest::Result<reqwest::Response> {
/// #   let client = reqwest::Client::new();
/// #   
/// #   client
/// #     .post("http://127.0.0.1:8080/webhook")
/// #     .header("Content-Type", "application/json")
/// #     .header("Authorization", env!("TOPGG_WEBHOOK_PASSWORD"))
/// #     .body("{\"bot\":\"1026525568344264724\",\"user\":\"661200758510977084\",\"type\":\"test\",\"isWeekend\":false}")
/// #     .send()
/// #     .await
/// # }
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
/// async fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyVoteHandler {});
///
///   let router = Router::new().route("/", get(index)).nest(
///     "/webhook",
///     topgg::axum::webhook(env!("TOPGG_WEBHOOK_PASSWORD").to_string(), Arc::clone(&state)),
///   );
///
///   let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
///
/// # let local_immature_thread_closure = Arc::new(Notify::const_new());
/// # let thread_immature_thread_closure = Arc::clone(&local_immature_thread_closure);
/// #
/// # let (shutdown_tx, shutdown_rx) = oneshot::channel();
/// #
/// # let test_thread = tokio::spawn(async move {
/// #   sleep(Duration::from_secs(5));
/// #   
/// #   tokio::select! {
/// #     _ = thread_immature_thread_closure.notified() => {
/// #       return Ok(None);
/// #     }
/// #     
/// #     result = test_request() => {
/// #       shutdown_tx.send(()).unwrap();
/// #       
/// #       return result.map(Some);
/// #     }
/// #   }
/// # });
/// #
/// # if let Err(why) =
///   axum::serve(listener, router)
/// #   .with_graceful_shutdown(async { shutdown_rx.await.ok(); })
///     .await
/// # {
/// #   local_immature_thread_closure.notify_one();
/// #   
/// #   panic!("Server error: {why:?}");
/// # }
/// # /*
///     .unwrap();
/// # */
/// #
/// # let test_response = test_thread.await.unwrap().unwrap().unwrap();
/// #
/// # assert_eq!(test_response.status(), reqwest::StatusCode::NO_CONTENT);
/// }
/// ```
#[inline(always)]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
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
