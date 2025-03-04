use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

util::debug_struct! {
  /// A Top.gg voter.
  #[must_use]
  #[derive(Clone, Deserialize)]
  Voter {
    public {
      /// This voter's Discord ID.
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// This voter's username.
      #[serde(rename = "username")]
      name: String,

      /// This voter's avatar URL.
      avatar: String,
    }

    getters(self) {
      /// This voter's creation date.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
      }
    }
  }
}
