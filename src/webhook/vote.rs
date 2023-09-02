use crate::snowflake;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// A struct representing a dispatched [Top.gg](https://top.gg) bot/server vote event.
#[must_use]
#[cfg_attr(docsrs, doc(cfg(feature = "webhook")))]
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
  #[serde(
    default = "_true",
    deserialize_with = "deserialize_is_server",
    rename = "bot"
  )]
  pub is_server: bool,

  /// Whether this vote is just a test coming from the bot/server owner or not. Most of the time this would be `false`.
  #[serde(deserialize_with = "deserialize_is_test", rename = "type")]
  pub is_test: bool,

  /// Whether the weekend multiplier is active or not, meaning a single vote counts as two.
  /// If the dispatched event came from a server being voted, this will always be `false`.
  #[serde(default, rename = "isWeekend")]
  pub is_weekend: bool,

  /// Query strings found on the vote page.
  #[serde(default, deserialize_with = "deserialize_query_string")]
  pub query: HashMap<String, String>,
}

#[inline(always)]
fn deserialize_is_test<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer).map(|s| s == "test")
}

const fn _true() -> bool {
  true
}

#[inline(always)]
fn deserialize_is_server<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(String::deserialize(deserializer).is_err())
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

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix-web", feature = "rocket"))] {
    /// A struct that represents an **unauthenticated** request containing a [`Vote`] data.
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
      /// Basic usage with [`actix-web`](https://actix.rs/):
      ///
      /// ```rust,no_run
      /// use actix_web::{
      ///   error::{Error, ErrorUnauthorized},
      ///   get, post,
      ///   App, HttpServer,
      /// };
      /// use std::io;
      /// use topgg::IncomingVote;
      ///
      /// #[get("/")]
      /// async fn index() -> &'static str {
      ///   "Hello, World!"
      /// }
      ///
      /// #[post("/webhook")]
      /// async fn webhook(vote: IncomingVote) -> Result<&'static str, Error> {
      ///   match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
      ///     Some(vote) => {
      ///       println!("{:?}", vote);
      ///
      ///       Ok("ok")
      ///     },
      ///     _ => Err(ErrorUnauthorized("401")),
      ///   }
      /// }
      ///
      /// #[actix_web::main]
      /// async fn main() -> io::Result<()> {
      ///   HttpServer::new(|| App::new().service(index).service(webhook))
      ///     .bind("127.0.0.1:8080")?
      ///     .run()
      ///     .await
      /// }
      /// ```
      ///
      /// Basic usage with [`rocket`](https://rocket.rs):
      ///
      /// ```rust,no_run
      /// #![feature(decl_macro)]
      ///
      /// use rocket::{get, http::Status, post, routes};
      /// use topgg::IncomingVote;
      ///
      /// #[get("/")]
      /// fn index() -> &'static str {
      ///   "Hello, World!"
      /// }
      ///
      /// #[post("/webhook", data = "<vote>")]
      /// fn webhook(vote: IncomingVote) -> Status {
      ///   match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
      ///     Some(vote) => {
      ///       println!("{:?}", vote);
      ///
      ///       // SAFETY: 200 and 401 will always be a valid status code.
      ///       unsafe { Status::from_code(200).unwrap_unchecked() }
      ///     },
      ///     _ => {
      ///       println!("found an unauthorized attacker.");
      ///
      ///       unsafe { Status::from_code(401).unwrap_unchecked() }
      ///     },
      ///   }
      /// }
      ///
      /// fn main() {
      ///   rocket::ignite()
      ///     .mount("/", routes![index, webhook])
      ///     .launch();
      /// }
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
    /// ```rust,no_run
    /// #[async_trait::async_trait]
    /// pub trait VoteHandler: Send + Sync + 'static {
    ///   async fn voted(&self, vote: Vote);
    /// }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(any(feature = "axum", feature = "warp"))))]
    #[async_trait::async_trait]
    pub trait VoteHandler: Send + Sync + 'static {
      /// Your vote handler's on-vote async callback. The endpoint will always return a 200 (OK) HTTP status code after running this method.
      ///
      /// # Examples
      ///
      /// Basic usage with [`axum`](https://crates.io/crates/axum):
      ///
      /// ```rust,no_run
      /// use axum::{routing::get, Router, Server};
      /// use std::sync::Arc;
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
      ///   let app = Router::new()
      ///     .route("/", get(index))
      ///     .nest("/webhook", topgg::axum::webhook(env!("TOPGG_WEBHOOK_PASSWORD").to_string(), state.clone()));
      ///
      ///   // SAFETY: this will always be a valid SocketAddr syntax.
      ///   let addr = unsafe { "127.0.0.1:8080".parse().unwrap_unchecked() };
      ///
      ///   Server::bind(&addr)
      ///     .serve(app.into_make_service())
      ///     .await
      ///     .unwrap();
      /// }
      /// ```
      ///
      /// Basic usage with [`warp`](https://crates.io/crates/warp):
      ///
      /// ```rust,no_run
      /// use std::{net::SocketAddr, sync::Arc};
      /// use topgg::{Vote, VoteHandler};
      /// use warp::Filter;
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
      ///   let webhook = topgg::warp::webhook("webhook", env!("TOPGG_WEBHOOK_PASSWORD").to_string(), state.clone());
      ///
      ///   let routes = warp::get()
      ///     .map(|| "Hello, World!")
      ///     .or(webhook);
      ///
      ///   // SAFETY: this will always be a valid SocketAddr syntax.
      ///   let addr: SocketAddr = unsafe { "127.0.0.1:8080".parse().unwrap_unchecked() };
      ///
      ///   warp::serve(routes)
      ///     .run(addr)
      ///     .await
      /// }
      /// ```
      async fn voted(&self, vote: Vote);
    }
  }
}
