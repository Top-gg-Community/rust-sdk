use crate::{Vote, VoteHandler};
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, Filter, Rejection, Reply};

/// Creates a new warp [`Filter`] for adding an on-vote event handler to your application logic.
///
/// # Example
///
/// Basic usage:
///
/// ```rust
/// use std::sync::Arc;
/// use tokio::net::TcpListener;
/// use topgg::{Vote, VoteHandler};
/// use warp::Filter;
/// #
/// # use std::time::Duration;
/// # use tokio::{sync::oneshot, time::sleep};
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
///   let webhook = topgg::warp::webhook(
///     "webhook",
///     env!("TOPGG_WEBHOOK_PASSWORD").to_string(),
///     Arc::clone(&state),
///   );
///
///   let routes = warp::get().map(|| "Hello, World!").or(webhook);
///
///   let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
/// #
/// # let (shutdown_tx, shutdown_rx) = oneshot::channel();
/// #
/// # let test_thread = tokio::spawn(async move {
/// #   sleep(Duration::from_secs(5));
/// #   
/// #   let client = reqwest::Client::new();
/// #   
/// #   let response = client
/// #     .post("http://127.0.0.1:8080/webhook")
/// #     .header("Content-Type", "application/json")
/// #     .header("Authorization", env!("TOPGG_WEBHOOK_PASSWORD"))
/// #     .body("{\"bot\":\"1026525568344264724\",\"user\":\"661200758510977084\",\"type\":\"test\",\"isWeekend\":false}")
/// #     .send()
/// #     .await;
/// #   
/// #   shutdown_tx.send(()).unwrap();
/// #   
/// #   response
/// # });
///
///   warp::serve(routes)
///     .incoming(listener)
/// #   .graceful(async { shutdown_rx.await.ok(); })
///     .run()
///     .await;
/// #
/// # let test_response = test_thread.await.unwrap().unwrap();
/// #
/// # assert_eq!(test_response.status(), reqwest::StatusCode::NO_CONTENT);
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
      let current_state = Arc::clone(&state);
      let current_password = Arc::clone(&password);

      async move {
        if auth == *current_password {
          current_state.voted(vote).await;

          StatusCode::NO_CONTENT
        } else {
          StatusCode::UNAUTHORIZED
        }
      }
    })
}
