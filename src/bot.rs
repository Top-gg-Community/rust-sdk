use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[inline(always)]
pub(crate) fn deserialize_support_server<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  util::deserialize_optional_string(deserializer)
    .map(|inner| inner.map(|support| format!("https://discord.com/invite/{support}")))
}

util::debug_struct! {
  /// A struct representing a Discord Bot listed on Top.gg.
  #[must_use]
  #[derive(Clone, Deserialize)]
  Bot {
    public {
      /// This bot's Discord ID.
      #[serde(rename = "clientid", deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// This bot's Top.gg ID.
      #[serde(rename = "id", deserialize_with = "snowflake::deserialize")]
      topgg_id: u64,

      /// This bot's username.
      username: String,

      /// This bot's discriminator.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      discriminator: String,

      /// This bot's prefix.
      prefix: String,

      /// This bot's short description.
      #[serde(rename = "shortdesc")]
      short_description: String,

      /// This bot's HTML/Markdown long description.
      #[serde(
        default,
        deserialize_with = "util::deserialize_optional_string",
        rename = "longdesc"
      )]
      long_description: Option<String>,

      /// This bot's tags.
      #[serde(default, deserialize_with = "util::deserialize_default")]
      tags: Vec<String>,

      /// This bot's website URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      website: Option<String>,

      /// This bot's GitHub repository URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      github: Option<String>,

      /// This bot's owner IDs.
      #[serde(deserialize_with = "snowflake::deserialize_vec")]
      owners: Vec<u64>,

      /// This bot's guild IDs.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      guilds: Vec<u64>,

      /// This bot's banner image URL.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      banner_url: Option<String>,

      /// This bot's approval date.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "Actually refers to submission date. Use `submitted_at` instead.")]
      approved_at: DateTime<Utc>,

      /// This bot's submission date.
      #[serde(rename = "date")]
      submitted_at: DateTime<Utc>,

      /// Whether this bot is certified or not.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      is_certified: bool,

      /// This bot's shards.
      #[serde(default, deserialize_with = "util::deserialize_deprecated")]
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      shards: Vec<usize>,

      /// The amount of votes this bot has.
      #[serde(rename = "points")]
      votes: usize,

      /// The amount of votes this bot has this month.
      #[serde(rename = "monthlyPoints")]
      monthly_votes: usize,

      /// This bot's support URL.
      #[serde(default, deserialize_with = "deserialize_support_server")]
      support: Option<String>,

      /// This bot's avatar URL.
      avatar: String,

      /// This bot's Top.gg vanity code.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      vanity: Option<String>,

      /// This bot's posted server count.
      #[serde(default)]
      server_count: Option<usize>,
    }

    private {
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      invite: Option<String>,
    }

    getters(self) {
      /// This bot's creation date.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
      }

      /// This bot's avatar URL.
      #[deprecated(since = "1.5.0", note = "Just directly use the public `avatar` property.")]
      avatar: String => {
        self.avatar.clone()
      }

      /// This bot's invite URL.
      #[must_use]
      invite: String => {
        match &self.invite {
          Some(inv) => inv.to_owned(),
          _ => format!(
            "https://discord.com/oauth2/authorize?scope=bot&client_id={}",
            self.id
          ),
        }
      }

      /// This bot's shard count.
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      shard_count: usize => {
        0
      }

      /// This bot's Top.gg page URL.
      #[must_use]
      #[inline(always)]
      url: String => {
        format!(
          "https://top.gg/bot/{}",
          self.vanity.as_deref().unwrap_or(&self.id.to_string())
        )
      }
    }
  }
}

util::debug_struct! {
  /// A struct representing a Discord bot's statistics.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use topgg::Stats;
  ///
  /// let _stats = Stats {
  ///   server_count: Some(12345),
  /// };
  /// ```
  #[must_use]
  #[derive(Clone, Serialize, Deserialize)]
  Stats {
    public {
      /// The amount of servers this bot is in. `None` if such information is publicly unavailable.
      #[serde(skip_serializing_if = "Option::is_none")]
      server_count: Option<usize>,
    }

    getters(self) {
      /// This bot's list of server count for each shard.
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      shards: &[usize] => {
        &[]
      }

      /// This bot's shard count.
      #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
      shard_count: usize => {
        0
      }

      /// The amount of servers this bot is in. `None` if such information is publicly unavailable.
      #[deprecated(since = "1.5.0", note = "Just directly use the public `server_count` property.")]
      server_count: Option<usize> => {
        self.server_count
      }
    }
  }
}

impl Stats {
  /// Creates a [`Stats`] struct from the cache of a serenity [`Context`][serenity::client::Context].
  #[inline(always)]
  #[cfg(feature = "serenity-cached")]
  #[cfg_attr(docsrs, doc(cfg(feature = "serenity-cached")))]
  pub fn from_context(context: &serenity::client::Context) -> Self {
    Self {
      server_count: Some(context.cache.guilds().len()),
    }
  }

  /// Creates a [`Stats`] struct based on total server and optionally, shard count data.
  #[deprecated(since = "1.5.0", note = "Just directly use a struct declaration.")]
  pub const fn from_count(server_count: usize, _shard_count: Option<usize>) -> Self {
    Self {
      server_count: Some(server_count),
    }
  }

  /// Creates a [`Stats`] struct based on an array of server count per shard and optionally the index (to the array) of shard posting this data.
  #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
  pub fn from_shards<A>(shards: A, _shard_index: Option<usize>) -> Self
  where
    A: IntoIterator<Item = usize>,
  {
    Self {
      server_count: Some(shards.into_iter().sum()),
    }
  }
}

/// Creates a [`Stats`] struct solely from a server count.
impl From<usize> for Stats {
  #[inline(always)]
  fn from(server_count: usize) -> Self {
    Self {
      server_count: Some(server_count),
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct IsWeekend {
  pub(crate) is_weekend: bool,
}
