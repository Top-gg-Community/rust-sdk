use crate::snowflake;
use serde::{
  de::{self, Deserializer},
  Deserialize,
};
use std::collections::HashMap;

/// A struct representing a dispatched [top.gg](https://top.gg) bot/server vote event.
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

  /// Whether this vote is just a test coming from the bot/server owner or not. Most of the time this would be `true`.
  #[serde(deserialize_with = "deserialize_is_test", rename = "type")]
  pub is_test: bool,

  /// Whether the weekend multiplier is active or not, meaning a single vote counts as two.
  /// If the dispatched event came from a server being voted, this will always be `false`.
  #[serde(
    default = "_false",
    deserialize_with = "deserialize_is_weekend",
    rename = "isWeekend"
  )]
  pub is_weekend: bool,

  /// Query strings found on the vote page, if any.
  #[serde(default = "_none", deserialize_with = "deserialize_query_string")]
  pub query: Option<HashMap<String, String>>,
}

#[inline(always)]
fn deserialize_is_test<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  let s: &str = de::Deserialize::deserialize(deserializer)?;

  Ok(s == "test")
}

const fn _false() -> bool {
  false
}

#[inline(always)]
fn deserialize_is_weekend<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(de::Deserialize::deserialize(deserializer).unwrap_or(_false()))
}

const fn _none<T>() -> Option<T> {
  None
}

fn deserialize_query_string<'de, D>(
  deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
  D: Deserializer<'de>,
{
  let s: Result<&str, D::Error> = de::Deserialize::deserialize(deserializer);

  Ok(match s {
    Ok(s) => {
      let mut output = HashMap::new();

      for mut it in s.split('&').map(|pair| pair.split('=')) {
        if let (Some(k), Some(v)) = (it.next(), it.next()) {
          match urlencoding::decode(v) {
            Ok(v) => output.insert(k.to_owned(), v.into_owned()),
            _ => continue,
          };
        }
      }

      Some(output)
    }

    _ => _none(),
  })
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix", feature = "rocket"))] {
    /// A struct that represents an unauthenticated request containing a [`Vote`] data.
    pub struct IncomingVote {
      pub(crate) authorization: String,
      pub(crate) vote: Vote,
    }

    impl IncomingVote {
      /// Authenticates a valid password with this request.
      /// Returns [`Some(Vote)`][`Vote`] if succeeds, otherwise `None`.
      #[must_use]
      #[inline(always)]
      pub fn authenticate<S>(self, password: &S) -> Option<Vote>
      where
        S: AsRef<str> + ?Sized,
      {
        if self.authorization == password.as_ref() {
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
    /// It's described as follows (without `async_trait`'s macro expansion):
    /// ```rust,no_run
    /// #[async_trait::async_trait]
    /// pub trait VoteHandler: Send + Sync + 'static {
    ///   async fn voted(&self, vote: Vote);
    /// }
    /// ```
    #[async_trait::async_trait]
    pub trait VoteHandler: Send + Sync + 'static {
      async fn voted(&self, vote: Vote);
    }
  }
}
