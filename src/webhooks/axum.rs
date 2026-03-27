use super::Payload;
use std::sync::Arc;

use axum::{
  Router,
  extract::State,
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Response},
  routing::post,
};

/// An axum webhook listener for listening to payloads.
///
/// # Example
///
/// ```rust,no_run
/// struct MyTopggListener {}
///
/// #[async_trait::async_trait]
/// impl topgg::axum::Listener for MyTopggListener {
///   async fn callback(self: Arc<Self>, payload: Payload, _trace: &str) -> Response {
///     println!("{payload:?}");
///
///     (StatusCode::NO_CONTENT, ()).into_response()
///   }
/// }
/// ```
#[async_trait::async_trait]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
pub trait Listener: Send + Sync + 'static {
  async fn callback(self: Arc<Self>, payload: Payload, trace: &str) -> Response;
}

struct WebhookState<T> {
  state: Arc<T>,
  secret: Arc<String>,
}

impl<T> Clone for WebhookState<T> {
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
      secret: self.secret.clone(),
    }
  }
}

/// Creates a new axum [`Router`] for receiving webhook payloads.
///
/// # Example
///
/// ```rust,no_run
/// use topgg::Payload;
/// use std::sync::Arc;
///
/// use axum::{http::status::StatusCode, response::{IntoResponse, Response}, routing::get, Router};
/// use tokio::net::TcpListener;
///
/// struct MyTopggListener {}
///
/// #[async_trait::async_trait]
/// impl topgg::axum::Listener for MyTopggListener {
///   async fn callback(self: Arc<Self>, payload: Payload, _trace: &str) -> Response {
///     println!("{payload:?}");
///
///     (StatusCode::NO_CONTENT, ()).into_response()
///   }
/// }
///
/// async fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyTopggListener {});
///
///   let router = Router::new().route("/", get(index)).nest(
///     "/webhook",
///     topgg::axum::webhook(Arc::clone(&state), env!("TOPGG_WEBHOOK_SECRET").to_string()),
///   );
///
///   let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
///
///   axum::serve(listener, router).await.unwrap();
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
pub fn webhook<S>(state: Arc<S>, secret: String) -> Router
where
  S: Listener,
{
  Router::new()
    .route(
      "/",
      post(
        async |headers: HeaderMap, State(wrapped_state): State<WebhookState<S>>, body: String| {
          if let Some(signature) = headers.get("x-topgg-signature")
            && let Ok(signature) = signature.to_str()
            && let Some(trace) = headers.get("x-topgg-trace")
            && let Ok(trace) = trace.to_str()
            && let Some(payload) = Payload::new(signature, &body, &wrapped_state.secret)
          {
            wrapped_state.state.callback(payload, trace).await
          } else {
            (StatusCode::UNAUTHORIZED, ()).into_response()
          }
        },
      ),
    )
    .with_state(WebhookState {
      state,
      secret: Arc::new(secret),
    })
}
