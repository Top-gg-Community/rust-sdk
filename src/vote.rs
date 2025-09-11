use crate::snowflake;
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A Top.gg vote.
#[derive(Deserialize)]
pub struct Vote {
  /// When the vote was cast.
  #[serde(rename = "created_at")]
  pub voted_at: DateTime<Utc>,

  /// When the vote expires and the user is required to vote again.
  pub expires_at: DateTime<Utc>,

  /// This vote's weight.
  pub weight: usize,
}

impl Vote {
  /// Whether this vote is now expired.
  #[inline(always)]
  pub fn expired(&self) -> bool {
    Utc::now() >= self.expires_at
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

/// A Top.gg voter.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Voter {
  /// This voter's ID.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// This voter's username.
  #[serde(rename = "username")]
  pub name: String,

  /// This voter's avatar URL.
  pub avatar: String,
}
