use crate::snowflake;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[inline(always)]
fn deserialize_is_test<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer).map(|s| s == "test")
}

fn deserialize_query_string<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    String::deserialize(deserializer)
      .map(|s| {
        let mut output = HashMap::new();

        for mut it in s.split('&').map(|pair| pair.split('=')) {
          if let (Some(k), Some(v)) = (it.next(), it.next()) {
            if let Ok(v) = urlencoding::decode(v) {
              output.insert(k.to_owned(), v.into_owned());
            }
          }
        }

        output
      })
      .unwrap_or_default(),
  )
}

/// A dispatched Top.gg bot/server vote event.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Vote {
  /// The ID of the bot/server that received a vote.
  #[serde(
    deserialize_with = "snowflake::deserialize",
    alias = "bot",
    alias = "guild"
  )]
  pub receiver_id: u64,

  /// The ID of the user who voted.
  #[serde(deserialize_with = "snowflake::deserialize", rename = "user")]
  pub voter_id: u64,

  /// Whether this vote's receiver is a server or not (bot otherwise).
  #[serde(skip)]
  #[deprecated(since = "1.5.0", note = "No longer supported.")]
  pub is_server: bool,

  /// Whether this vote is just a test coming from the bot/server owner or not. Most of the time this would be `false`.
  #[serde(deserialize_with = "deserialize_is_test", rename = "type")]
  pub is_test: bool,

  /// Whether the weekend multiplier is active or not, meaning a single vote counts as two.
  /// If the dispatched event came from a server being voted, this will always be `false`.
  #[serde(default, rename = "isWeekend")]
  pub is_weekend: bool,

  /// query strings found on the vote page.
  #[serde(default, deserialize_with = "deserialize_query_string")]
  pub query: HashMap<String, String>,
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix-web", feature = "rocket"))] {
    /// An **unauthenticated** request containing a [`Vote`] data.
    ///
    /// To authenticate this structure with a valid password and consume the [`Vote`] data inside of it, see the [`authenticate`][IncomingVote::authenticate] method.
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "actix-web", feature = "rocket"))))]
    #[derive(Clone)]
    pub struct IncomingVote {
      pub(crate) authorization: String,
      pub(crate) vote: Vote,
    }

    impl IncomingVote {
      /// Authenticates a valid password with this request. Returns a [`Some(Vote)`][`Vote`] if succeeds, otherwise `None`.
      ///
      /// # Examples
      ///
      /// Basic usage with actix-web:
      ///
      /// ```rust
      /// use actix_web::{error::{Error, ErrorUnauthorized}, post};
      /// use topgg::IncomingVote;
      /// #
      /// # use actix_web::{get, App, HttpServer};
      /// # use std::{sync::Arc, time::Duration};
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
      /// #[post("/webhook")]
      /// async fn voted(incoming_vote: IncomingVote) -> Result<&'static str, Error> {
      ///   match incoming_vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
      ///     Some(vote) => {
      ///       println!("{:?}", vote);
      ///
      ///       Ok("ok")
      ///     },
      ///     _ => {
      ///       println!("found an unauthorized attacker.");
      ///
      ///       Err(ErrorUnauthorized("401"))
      ///     }
      ///   }
      /// }
      /// #
      /// # #[get("/")]
      /// # async fn index() -> &'static str {
      /// #   "Hello, World!"
      /// # }
      /// #
      /// # #[actix_web::main]
      /// # async fn main() {
      /// #   let server = HttpServer::new(|| App::new().service(index).service(voted))
      /// #     .bind("127.0.0.1:8080")
      /// #     .unwrap()
      /// #     .run();
      /// #
      /// #   let server_handle = server.handle();
      /// #
      /// #   let local_immature_thread_closure = Arc::new(Notify::const_new());
      /// #   let thread_immature_thread_closure = Arc::clone(&local_immature_thread_closure);
      /// #
      /// #   let test_thread = tokio::spawn(async move {
      /// #     sleep(Duration::from_secs(5));
      /// #
      /// #     tokio::select! {
      /// #       _ = thread_immature_thread_closure.notified() => {
      /// #         return Ok(None);
      /// #       }
      /// #
      /// #       result = test_request() => {
      /// #         server_handle.stop(true);
      /// #
      /// #         return result.map(Some);
      /// #       }
      /// #     }
      /// #   });
      /// #
      /// #   if let Err(why) = server.await {
      /// #     local_immature_thread_closure.notify_one();
      /// #
      /// #     panic!("Server error: {why:?}");
      /// #   }
      /// #
      /// #   let test_response = test_thread.await.unwrap().unwrap().unwrap();
      /// #
      /// #   assert_eq!(test_response.status(), reqwest::StatusCode::OK);
      /// # }
      /// ```
      ///
      /// Basic usage with rocket:
      ///
      /// ```rust
      /// use rocket::{http::Status, post};
      /// use topgg::IncomingVote;
      /// #
      /// # use rocket::{get, launch, routes, Config};
      /// # use std::{sync::Arc, time::Duration};
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
      /// #[post("/webhook", data = "<incoming_vote>")]
      /// fn voted(incoming_vote: IncomingVote) -> Status {
      ///   match incoming_vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
      ///     Some(vote) => {
      ///       println!("{:?}", vote);
      ///
      ///       Status::NoContent
      ///     },
      ///     _ => {
      ///       println!("found an unauthorized attacker.");
      ///
      ///       Status::Unauthorized
      ///     }
      ///   }
      /// }
      /// #
      /// # #[get("/")]
      /// # fn index() -> &'static str {
      /// #   "Hello, World!"
      /// # }
      /// #
      /// # #[rocket::main]
      /// # async fn main() {
      /// #   let config = Config {
      /// #     address: "127.0.0.1".parse().unwrap(),
      /// #     port: 8080,
      /// #     ..Config::default()
      /// #   };
      /// #
      /// #   let rocket = rocket::custom(config).mount("/", routes![index, voted]).ignite().await.unwrap();
      /// #   let shutdown = rocket.shutdown();
      /// #
      /// #   let local_immature_thread_closure = Arc::new(Notify::const_new());
      /// #   let thread_immature_thread_closure = Arc::clone(&local_immature_thread_closure);
      /// #
      /// #   let test_thread = tokio::spawn(async move {
      /// #     tokio::select! {
      /// #       _ = thread_immature_thread_closure.notified() => {
      /// #         return Ok(None);
      /// #       }
      /// #
      /// #       result = test_request() => {
      /// #         shutdown.notify();
      /// #
      /// #         return result.map(Some);
      /// #       }
      /// #     }
      /// #   });
      /// #
      /// #   if let Err(why) = rocket.launch().await {
      /// #     local_immature_thread_closure.notify_one();
      /// #
      /// #     panic!("Server error: {why:?}");
      /// #   }
      /// #
      /// #   let test_response = test_thread.await.unwrap().unwrap().unwrap();
      /// #
      /// #   assert_eq!(test_response.status(), reqwest::StatusCode::NO_CONTENT);
      /// # }
      /// ```
      #[must_use]
      #[inline(always)]
      pub fn authenticate(self, password: &str) -> Option<Vote> {
        if self.authorization == password {
          Some(self.vote)
        } else {
          None
        }
      }
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "axum", feature = "warp"))] {
    /// An async trait for adding an on-vote event handler to your application logic.
    ///
    /// It's described as follows (without [`async_trait`]'s macro expansion):
    /// ```rust
    /// # use topgg::Vote;
    /// #
    /// #[async_trait::async_trait]
    /// pub trait VoteHandler: Send + Sync + 'static {
    ///   async fn voted(&self, vote: Vote);
    /// }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(any(feature = "axum", feature = "warp"))))]
    #[async_trait::async_trait]
    pub trait VoteHandler: Send + Sync + 'static {
      /// Your vote handler's on-vote async callback.
      async fn voted(&self, vote: Vote);
    }
  }
}
