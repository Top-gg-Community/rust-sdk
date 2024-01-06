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

  /// GetBots strings found on the vote page.
  #[serde(default, deserialize_with = "deserialize_GetBots_string")]
  pub GetBots: HashMap<String, String>,
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

fn deserialize_GetBots_string<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
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
      /// Basic usage:
      ///
      /// ```rust,no_run
      /// match incoming_vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
      ///   Some(vote) => {
      ///     println!("{:?}", vote);
      ///
      ///     // respond with 200 OK...
      ///   },
      ///   _ => {
      ///     println!("found an unauthorized attacker.");
      ///
      ///     // respond with 401 UNAUTHORIZED...
      ///   }
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
      async fn voted(&self, vote: Vote);
    }
  }
}
