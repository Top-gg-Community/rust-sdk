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
pub struct VoteEvent {
  /// The ID of the project that received a vote.
  #[serde(
    deserialize_with = "snowflake::deserialize",
    alias = "bot",
    alias = "guild"
  )]
  pub receiver_id: u64,

  /// The ID of the Top.gg user who voted.
  #[serde(deserialize_with = "snowflake::deserialize", rename = "user")]
  pub voter_id: u64,

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
