use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A user's social links.
#[allow(clippy::doc_markdown)]
#[derive(Clone, Debug, Deserialize)]
#[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
pub struct Socials {
  /// This user's GitHub account URL.
  #[serde(skip)]
  pub github: Option<String>,

  /// This user's Instagram account URL.
  #[serde(skip)]
  pub instagram: Option<String>,

  /// This user's Reddit account URL.
  #[serde(skip)]
  pub reddit: Option<String>,

  /// This user's Twitter account URL.
  #[serde(skip)]
  pub twitter: Option<String>,

  /// This user's YouTube channel URL.
  #[serde(skip)]
  pub youtube: Option<String>,
}

util::debug_struct! {
  /// A user logged into Top.gg.
  #[derive(Clone, Deserialize)]
  #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
  User {
    public {
      /// This user's ID.
      #[serde(skip)]
      id: u64,

      /// This user's username.
      #[serde(skip)]
      username: String,

      /// The user's bio.
      #[serde(skip)]
      bio: Option<String>,

      /// This user's profile banner image.
      #[serde(skip)]
      banner: Option<String>,

      /// This user's social links.
      #[serde(skip)]
      socials: Option<Socials>,

      /// Whether this user is a Top.gg supporter or not.
      #[serde(skip)]
      is_supporter: bool,

      /// Whether this user is a Top.gg certified developer or not.
      #[serde(skip)]
      is_certified_dev: bool,

      /// Whether this user is a Top.gg moderator or not.
      #[serde(skip)]
      is_moderator: bool,

      /// Whether this user is a Top.gg website moderator or not.
      #[serde(skip)]
      is_web_moderator: bool,

      /// Whether this user is a Top.gg website administrator or not.
      #[serde(skip)]
      is_admin: bool,
    }

    getters(self) {
      /// This user's creation date.
      #[allow(clippy::missing_panics_doc)]
      created_at: DateTime<Utc> => {
        panic!("The User struct is deprecated as it's no longer supported by API v0.")
      }

      /// This user's avatar URL.
      #[allow(clippy::missing_panics_doc)]
      avatar: String => {
        panic!("The User struct is deprecated as it's no longer supported by API v0.")
      }
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

util::debug_struct! {
  /// A user who has voted on a Discord bot listed on Top.gg. (See [`Client::get_voters`][crate::Client::get_voters])
  #[must_use]
  #[derive(Clone, Deserialize)]
  Voter {
    public {
      /// This voter's ID.
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// This voter's username.
      username: String,

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

      /// This voter's avatar URL.
      #[deprecated(since = "1.5.0", note = "Just directly use the public `avatar` property.")]
      avatar: String => {
        self.avatar.clone()
      }
    }
  }
}
