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

        for mut it in s
          .trim_start_matches('?')
          .split('&')
          .map(|pair| pair.split('='))
        {
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

/// A dispatched Top.gg vote event.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Vote {
  /// The ID of the Discord bot/server that received a vote.
  #[serde(
    deserialize_with = "snowflake::deserialize",
    alias = "bot",
    alias = "guild"
  )]
  pub receiver_id: u64,

  /// The ID of the Top.gg user who voted.
  #[serde(deserialize_with = "snowflake::deserialize", rename = "user")]
  pub voter_id: u64,

  /// Whether this vote's receiver is a Discord server.
  #[serde(
    default = "_true",
    deserialize_with = "deserialize_is_server",
    rename = "bot"
  )]
  pub is_server: bool,

  /// Whether this vote is just a test done from the page settings.
  #[serde(deserialize_with = "deserialize_is_test", rename = "type")]
  pub is_test: bool,

  /// Whether the weekend multiplier is active, where a single vote counts as two.
  #[serde(default, rename = "isWeekend")]
  pub is_weekend: bool,

  /// Query strings found on the vote page.
  #[serde(default, deserialize_with = "deserialize_query_string")]
  pub query: HashMap<String, String>,
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix-web", feature = "rocket"))] {
    /// An unauthenticated dispatched Top.gg vote event.
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "actix-web", feature = "rocket"))))]
    #[derive(Clone)]
    pub struct IncomingVote {
      pub(crate) authorization: String,
      pub(crate) vote: Vote,
    }

    impl IncomingVote {
      /// Authenticates a valid password with this request.
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
    /// Vote event handler.
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
      async fn voted(&self, vote: Vote);
    }
  }
}
