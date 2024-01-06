use crate::{snowflake, util, Client};
use chrono::{DateTime, Utc};
use core::{
  cmp::min,
  future::{Future, IntoFuture},
};
use serde::{Deserialize, Deserializer, Serialize};
use std::pin::Pin;

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
  /// A struct representing a Discord Bot listed on [Top.gg](https://top.gg).
  #[must_use]
  #[derive(Clone, Deserialize)]
  Bot {
    public {
      /// The ID of this Discord bot.
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,

      /// The username of this Discord bot.
      username: String,

      /// The discriminator of this Discord bot.
      discriminator: String,

      /// The prefix of this Discord bot.
      prefix: String,

      /// The short description of this Discord bot.
      #[serde(rename = "shortdesc")]
      short_description: String,

      /// The long description of this Discord bot. It can contain HTML and/or Markdown.
      #[serde(
        default,
        deserialize_with = "util::deserialize_optional_string",
        rename = "longdesc"
      )]
      long_description: Option<String>,

      /// The tags of this Discord bot.
      #[serde(default, deserialize_with = "util::deserialize_default")]
      tags: Vec<String>,

      /// The website URL of this Discord bot.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      website: Option<String>,

      /// The link to this Discord bot's GitHub repository.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      github: Option<String>,

      /// A list of IDs of this Discord bot's owners. The main owner is the first ID in the array.
      #[serde(deserialize_with = "snowflake::deserialize_vec")]
      owners: Vec<u64>,

      /// A list of IDs of the guilds featured on this Discord bot's page.
      #[serde(default, deserialize_with = "snowflake::deserialize_vec")]
      guilds: Vec<u64>,

      /// The URL for this Discord bot's banner image.
      #[serde(
        default,
        deserialize_with = "util::deserialize_optional_string",
        rename = "bannerUrl"
      )]
      banner_url: Option<String>,

      /// The date when this Discord bot was approved on [Top.gg](https://top.gg).
      #[serde(rename = "date")]
      approved_at: DateTime<Utc>,

      /// Whether this Discord bot is [Top.gg](https://top.gg) certified or not.
      #[serde(rename = "certifiedBot")]
      is_certified: bool,

      /// A list of this Discord bot's shards.
      #[serde(default, deserialize_with = "util::deserialize_default")]
      shards: Vec<usize>,

      /// The amount of upvotes this Discord bot has.
      #[serde(rename = "points")]
      votes: usize,

      /// The amount of upvotes this Discord bot has this month.
      #[serde(rename = "monthlyPoints")]
      monthly_votes: usize,

      /// The support server invite URL of this Discord bot.
      #[serde(default, deserialize_with = "deserialize_support_server")]
      support: Option<String>,
    }

    private {
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      avatar: Option<String>,

      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      invite: Option<String>,

      shard_count: Option<usize>,

      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      vanity: Option<String>,
    }

    getters(self) {
      /// Retrieves the creation date of this bot.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
      }

      /// Retrieves the avatar URL of this bot.
      ///
      /// Its format will either be PNG or GIF if animated.
      #[must_use]
      #[inline(always)]
      avatar: String => {
        util::get_avatar(&self.avatar, self.id)
      }

      /// The invite URL of this Discord bot.
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

      /// The amount of shards this Discord bot has according to posted stats.
      #[must_use]
      #[inline(always)]
      shard_count: usize => {
        self.shard_count.unwrap_or(self.shards.len())
      }

      /// Retrieves the URL of this Discord bot's [Top.gg](https://top.gg) page.
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

#[derive(Deserialize)]
pub(crate) struct Bots {
  pub(crate) results: Vec<Bot>,
}

util::debug_struct! {
  /// A struct representing a Discord bot's statistics.
  ///
  /// # Examples
  ///
  /// Solely from a server count:
  ///
  /// ```rust,no_run
  /// use topgg::Stats;
  ///
  /// let _stats = Stats::from(12345);
  /// ```
  ///
  /// Server count with a shard count:
  ///
  /// ```rust,no_run
  /// use topgg::Stats;
  ///
  /// let server_count = 12345;
  /// let shard_count = 10;
  /// let _stats = Stats::from_count(server_count, Some(shard_count));
  /// ```
  ///
  /// Solely from shards information:
  ///
  /// ```rust,no_run
  /// use topgg::Stats;
  ///
  /// // the shard posting this data has 456 servers.
  /// let _stats = Stats::from_shards([123, 456, 789], Some(1));
  /// ```
  #[must_use]
  #[derive(Clone, Serialize, Deserialize)]
  Stats {
    protected {
      shard_count: Option<usize>,
      server_count: Option<usize>,
    }

    private {
      #[serde(default, deserialize_with = "util::deserialize_default")]
      shards: Option<Vec<usize>>,
      #[serde(default, deserialize_with = "util::deserialize_default")]
      shard_id: Option<usize>,
    }

    getters(self) {
      /// An array of this Discord bot's server count for each shard.
      #[must_use]
      #[inline(always)]
      shards: &[usize] => {
        match self.shards {
          Some(ref shards) => shards,
          None => &[],
        }
      }

      /// The amount of shards this Discord bot has.
      #[must_use]
      #[inline(always)]
      shard_count: usize => {
        self.shard_count.unwrap_or(match self.shards {
          Some(ref shards) => shards.len(),
          None => 0,
        })
      }

      /// The amount of servers this bot is in. `None` if such information is publy unavailable.
      #[must_use]
      server_count: Option<usize> => {
        self.server_count.or_else(|| {
          self.shards.as_ref().and_then(|shards| {
            if shards.is_empty() {
              None
            } else {
              Some(shards.iter().copied().sum())
            }
          })
        })
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
    Self::from_count(
      context.cache.guilds().len(),
      Some(context.cache.shard_count() as _),
    )
  }

  /// Creates a [`Stats`] struct based on total server and optionally, shard count data.
  pub const fn from_count(server_count: usize, shard_count: Option<usize>) -> Self {
    Self {
      server_count: Some(server_count),
      shard_count,
      shards: None,
      shard_id: None,
    }
  }

  /// Creates a [`Stats`] struct based on an array of server count per shard and optionally the index (to the array) of shard posting this data.
  ///
  /// # Panics
  ///
  /// Panics if the shard_index argument is [`Some`] yet it's out of range of the `shards` array.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Stats;
  ///
  /// // the shard posting this data has 456 servers.
  /// let _stats = Stats::from_shards([123, 456, 789], Some(1));
  /// ```
  pub fn from_shards<A>(shards: A, shard_index: Option<usize>) -> Self
  where
    A: IntoIterator<Item = usize>,
  {
    let mut total_server_count = 0;
    let shards = shards.into_iter();
    let mut shards_list = Vec::with_capacity(shards.size_hint().0);

    for server_count in shards {
      total_server_count += server_count;
      shards_list.push(server_count);
    }

    if let Some(index) = shard_index {
      assert!(index < shards_list.len(), "Shard index out of range.");
    }

    Self {
      server_count: Some(total_server_count),
      shard_count: Some(shards_list.len()),
      shards: Some(shards_list),
      shard_id: shard_index,
    }
  }
}

/// Creates a [`Stats`] struct solely from a server count.
impl From<usize> for Stats {
  #[inline(always)]
  fn from(server_count: usize) -> Self {
    Self::from_count(server_count, None)
  }
}

#[derive(Deserialize)]
pub(crate) struct IsWeekend {
  pub(crate) is_weekend: bool,
}

/// A struct for configuring the query in [`get_bots`][crate::Client::get_bots] before being sent to the [Top.gg API](https://docs.top.gg) by `await`ing it.
#[must_use]
pub struct GetBots<'a> {
  client: &'a Client,
  query: String,
  search: String,
}

macro_rules! get_bots_method {
  ($(
    $(#[doc = $doc:literal])*
    $input_name:ident: $input_type:ty = $property:ident($($format:tt)*);
  )*) => {$(
    $(#[doc = $doc])*
    pub fn $input_name(mut self, $input_name: $input_type) -> Self {
      self.$property.push_str(&format!($($format)*));
      self
    }
  )*};
}

impl<'a> GetBots<'a> {
  #[inline(always)]
  pub(crate) fn new(client: &'a Client) -> Self {
    Self {
      client,
      query: String::from('?'),
      search: String::new(),
    }
  }

  get_bots_method! {
    /// Sets the maximum amount of bots to be queried.
    limit: u16 = query("limit={}&", min(limit, 500));

    /// Sets the amount of bots to be skipped during the query.
    skip: u16 = query("offset={}&", min(skip, 499));

    /// Queries only Discord bots that matches this username.
    username: &str = search("username%3A%20{}%20", urlencoding::encode(username));

    /// Queries only Discord bots that matches this discriminator.
    discriminator: &str = search("discriminator%3A%20{discriminator}%20");

    /// Queries only Discord bots that matches this prefix.
    prefix: &str = search("prefix%3A%20{}%20", urlencoding::encode(prefix));

    /// Queries only Discord bots that has this vote count.
    votes: usize = search("points%3A%20{votes}%20");

    /// Queries only Discord bots that has this monthly vote count.
    monthly_votes: usize = search("monthlyPoints%3A%20{monthly_votes}%20");

    /// Queries only [Top.gg](https://top.gg) certified Discord bots or not.
    certified: bool = search("certifiedBot%3A%20{certified}%20");

    /// Queries only Discord bots that has this [Top.gg](https://top.gg) vanity URL.
    vanity: &str = search("vanity%3A%20{}%20", urlencoding::encode(vanity));
  }
}

impl<'a> IntoFuture for GetBots<'a> {
  type Output = crate::Result<Vec<Bot>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

  fn into_future(self) -> Self::IntoFuture {
    let mut query = self.query;

    if !self.search.is_empty() {
      query.push_str(&format!("search={}", self.search));
    } else {
      query.pop();
    }

    Box::pin(self.client.get_bots_inner(query))
  }
}
