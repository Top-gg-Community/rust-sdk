use crate::snowflake;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

/// A Top.gg voter.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Voter {
  /// This voter's Discord ID.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// This voter's username.
  #[serde(rename = "username")]
  pub name: String,

  /// This voter's avatar URL.
  pub avatar: String,
}
