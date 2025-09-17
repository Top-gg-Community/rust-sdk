use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A struct representing a user's social links.
#[derive(Clone, Debug, Deserialize)]
pub struct Socials {
  /// A URL of this user's GitHub account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub github: Option<String>,

  /// A URL of this user's Instagram account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub instagram: Option<String>,

  /// A URL of this user's Reddit account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub reddit: Option<String>,

  /// A URL of this user's Twitter account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub twitter: Option<String>,

  /// A URL of this user's YouTube channel.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub youtube: Option<String>,
}

util::debug_struct! {
  /// A struct representing a user logged into [Top.gg](https://top.gg).
  #[must_use]
  #[derive(Clone, Deserialize)]
  User {
    public {
      /// The Discord ID of this user.
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// The username of this user.
      username: String,

      /// The user's bio.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      bio: Option<String>,

      /// A URL of this user's profile banner image.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      banner: Option<String>,

      /// A struct of this user's social links.
      #[serde(rename = "social")]
      socials: Option<Socials>,

      /// Whether this user is a [Top.gg](https://top.gg) supporter or not.
      #[serde(rename = "supporter")]
      is_supporter: bool,

      /// Whether this user is a [Top.gg](https://top.gg) certified developer or not.
      #[serde(rename = "certifiedDev")]
      is_certified_dev: bool,

      /// Whether this user is a [Top.gg](https://top.gg) moderator or not.
      #[serde(rename = "mod")]
      is_moderator: bool,

      /// Whether this user is a [Top.gg](https://top.gg) website moderator or not.
      #[serde(rename = "webMod")]
      is_web_moderator: bool,

      /// Whether this user is a [Top.gg](https://top.gg) website administrator or not.
      #[serde(rename = "admin")]
      is_admin: bool,
    }

    private {
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      avatar: Option<String>,
    }

    getters(self) {
      /// Retrieves the creation date of this user.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
      }

      /// Retrieves the Discord avatar URL of this user.
      ///
      /// Its format will either be PNG or GIF if animated.
      #[must_use]
      #[inline(always)]
      avatar: String => {
        util::get_avatar(&self.avatar, self.id)
      }
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

util::debug_struct! {
  /// A struct representing a user who has voted on a Discord bot listed on [Top.gg](https://top.gg). (See [`Client::get_voters`][crate::Client::get_voters])
  #[must_use]
  #[derive(Clone, Deserialize)]
  Voter {
    public {
      /// The Discord ID of this user.
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// The username of this user.
      username: String,
    }

    private {
      avatar: Option<String>,
    }

    getters(self) {
      /// Retrieves the creation date of this user.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
      }

      /// Retrieves the Discord avatar URL of this user.
      ///
      /// Its format will either be PNG or GIF if animated.
      #[must_use]
      #[inline(always)]
      avatar: String => {
        util::get_avatar(&self.avatar, self.id)
      }
    }
  }
}
