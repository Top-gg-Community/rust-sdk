use super::Webhook;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use warp::{body, header, http::StatusCode, path, Filter, Rejection, Reply};

/// Creates a new warp [`Filter`] for receiving webhook events.
///
/// # Example
///
/// ```rust,no_run
/// use std::{net::SocketAddr, sync::Arc};
/// use topgg::{VoteEvent, Webhook};
/// use warp::Filter;
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
/// #[tokio::main]
/// async fn main() {
///   let state = Arc::new(MyVoteListener {});
///
///   // POST /votes
///   let webhook = topgg::warp::webhook(
///     "votes",
///     env!("MY_TOPGG_WEBHOOK_SECRET").to_string(),
///     Arc::clone(&state),
///   );
///
///   let routes = warp::get().map(|| "Hello, World!").or(webhook);
///
///   let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///
///   warp::serve(routes).run(addr).await
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
pub fn webhook<D, T>(
  endpoint: &'static str,
  password: String,
  state: Arc<T>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone
where
  D: DeserializeOwned + Send,
  T: Webhook<D>,
{
  let password = Arc::new(password);

  warp::post()
    .and(path(endpoint))
    .and(header("Authorization"))
    .and(body::json())
    .then(move |auth: String, data: D| {
      let current_state = Arc::clone(&state);
      let current_password = Arc::clone(&password);

      async move {
        if auth == *current_password {
          current_state.callback(data).await;

          StatusCode::NO_CONTENT
        } else {
          StatusCode::UNAUTHORIZED
        }
      }
    })
}
