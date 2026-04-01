use super::{Payload, PayloadResult};
use std::{sync::Arc, time::Duration};

use axum::{
  BoxError, Router,
  error_handling::HandleErrorLayer,
  extract::{DefaultBodyLimit, State},
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Response},
  routing::post,
};
use chrono::Utc;
use tower::{ServiceBuilder, timeout::error::Elapsed};

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
/// use axum::{
///   Router,
///   http::status::StatusCode,
///   response::{IntoResponse, Response},
///   routing::get,
/// };
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
///   // POST /webhook
///   let router = Router::new().route("/", get(index)).nest(
///     "/webhook",
///     topgg::axum::webhook(Arc::clone(&state), env!("TOPGG_WEBHOOK_SECRET").into()),
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
  let timeout_layer = ServiceBuilder::new()
    .layer(HandleErrorLayer::new(|err: BoxError| async move {
      if err.is::<Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out")
      } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
      }
    }))
    .timeout(Duration::from_secs(5));

  Router::new()
    .route(
      "/",
      post(
        async |headers: HeaderMap, State(wrapped_state): State<WebhookState<S>>, body: String| {
          let now = Utc::now();

          if let Some(signature) = headers.get("x-topgg-signature")
            && let Ok(signature) = signature.to_str()
            && let Some(trace) = headers.get("x-topgg-trace")
            && let Ok(trace) = trace.to_str()
          {
            match Payload::new(now, body, signature, &wrapped_state.secret) {
              PayloadResult::Accepted(payload) => {
                wrapped_state.state.callback(payload, trace).await
              }

              PayloadResult::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),

              PayloadResult::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request").into_response(),

              PayloadResult::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
              }

              PayloadResult::DeserializationFailure => (StatusCode::NO_CONTENT, "").into_response(),

              PayloadResult::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
              }
            }
          } else {
            (StatusCode::BAD_REQUEST, "Bad Request").into_response()
          }
        },
      ),
    )
    .layer(timeout_layer.into_inner())
    .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
    .with_state(WebhookState {
      state,
      secret: Arc::new(secret),
    })
}
