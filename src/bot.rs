use crate::{snowflake, util, Client};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
  cmp::min,
  fmt::Write,
  future::{Future, IntoFuture},
  pin::Pin,
};

/// A Discord bot's reviews on Top.gg.
#[derive(Clone, Debug, Deserialize)]
pub struct BotReviews {
  /// This bot's average review score out of 5.
  #[serde(rename = "averageScore")]
  pub score: f64,

  /// This bot's review count.
  pub count: usize,
}

util::debug_struct! {
  /// A Discord Bot listed on Top.gg.
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
      #[serde(skip)]
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

      /// This bot's server IDs.
      #[serde(skip)]
      guilds: Vec<u64>,

      /// This bot's banner image URL.
      #[serde(skip)]
      banner_url: Option<String>,

      /// This bot's approval date.
      #[serde(skip)]
      #[deprecated(since = "1.5.0", note = "Actually refers to submission date. Use `submitted_at` instead.")]
      approved_at: DateTime<Utc>,

      /// This bot's submission date.
      #[serde(rename = "date")]
      submitted_at: DateTime<Utc>,

      /// Whether this bot is certified or not.
      #[serde(skip)]
      is_certified: bool,

      /// This bot's shards.
      #[serde(skip)]
      shards: Vec<usize>,

      /// The amount of votes this bot has.
      #[serde(rename = "points")]
      votes: usize,

      /// The amount of votes this bot has this month.
      #[serde(rename = "monthlyPoints")]
      monthly_votes: usize,

      /// This bot's support URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      support: Option<String>,

      /// This bot's avatar URL.
      avatar: String,

      /// This bot's Top.gg vanity code.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      vanity: Option<String>,

      /// This bot's posted server count.
      #[serde(default)]
      server_count: Option<usize>,

      /// This bot's reviews.
      #[serde(rename = "reviews")]
      review: BotReviews,
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

#[derive(Deserialize)]
pub(crate) struct Bots {
  pub(crate) results: Vec<Bot>,
}

util::debug_struct! {
  /// A Discord bot's statistics.
  ///
  /// # Examples
  ///
  /// Solely from a server count:
  ///
  /// ```rust
  /// use topgg::Stats;
  ///
  /// let _stats = Stats::from(12345);
  /// ```
  ///
  /// Server count with a shard count:
  ///
  /// ```rust
  /// use topgg::Stats;
  ///
  /// let server_count = 12345;
  /// let shard_count = 10;
  /// let _stats = Stats::from_count(server_count, Some(shard_count));
  /// ```
  ///
  /// Solely from shards information:
  ///
  /// ```rust
  /// use topgg::Stats;
  ///
  /// // the shard posting this data has 456 servers.
  /// let _stats = Stats::from_shards([123, 456, 789], Some(1));
  /// ```
  #[must_use]
  #[derive(Clone, Serialize, Deserialize)]
  Stats {
    protected {
      #[serde(skip_serializing_if = "Option::is_none")]
      shard_count: Option<usize>,

      #[serde(skip_serializing_if = "Option::is_none")]
      server_count: Option<usize>,
    }

    private {
      #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "util::deserialize_default")]
      shards: Option<Vec<usize>>,

      #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "util::deserialize_default")]
      shard_id: Option<usize>,
    }

    getters(self) {
      /// This bot's list of server count for each shard.
      #[must_use]
      #[inline(always)]
      shards: &[usize] => {
        match self.shards {
          Some(ref shards) => shards,
          None => &[],
        }
      }

      /// This bot's shard count.
      #[must_use]
      #[inline(always)]
      shard_count: usize => {
        self.shard_count.unwrap_or(match self.shards {
          Some(ref shards) => shards.len(),
          None => 0,
        })
      }

      /// The amount of servers this bot is in. `None` if such information is publicly unavailable.
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
  /// Panics if the `shard_index` argument is [`Some`] yet it's out of range of the `shards` array.
  ///
  /// # Example
  ///
  /// Basic usage:
  ///
  /// ```rust
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

/// Query configuration for [`get_bots`][crate::Client::get_bots].
#[must_use]
pub struct GetBots<'a> {
  client: &'a Client,
  query: String,
  search: String,
  sort: Option<&'static str>,
}

macro_rules! get_bots_method {
  ($(
    $(#[$details:meta])*
    $input_name:ident: $input_type:ty $(= $property:ident($($format:tt)*))?;
  )*) => {$(
    $(#[$details])*
    #[allow(unused, unused_mut)]
    pub fn $input_name(mut self, $input_name: $input_type) -> Self {
      $(write!(&mut self.$property, $($format)*).unwrap();)?
      self
    }
  )*};
}

macro_rules! get_bots_sort {
  ($(
    $(#[$details:meta])*
    $func_name:ident: $api_name:ident,
  )*) => {$(
    $(#[$details])*
    pub fn $func_name(mut self) -> Self {
      self.sort.replace(stringify!($api_name));
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
      sort: None,
    }
  }

  get_bots_sort! {
    /// Sorts results based on each bot's ID.
    sort_by_id: id,

    /// Sorts results based on each bot's approval date.
    sort_by_approval_date: date,

    /// Sorts results based on each bot's monthly vote count.
    sort_by_monthly_votes: monthlyPoints,
  }

  get_bots_method! {
    /// Sets the maximum amount of bots to be queried. This cannot be more than 500.
    limit: u16 = query("limit={}&", min(limit, 500));

    /// Sets the amount of bots to be skipped during the query. This cannot be more than 499.
    skip: u16 = query("offset={}&", min(skip, 499));

    /// Queries only bots that has this username.
    username: &str = search("username%3A%20{}%20", urlencoding::encode(username));

    /// Queries only bots that has this discriminator.
    discriminator: &str = search("discriminator%3A%20{discriminator}%20");

    /// Queries only bots that has this prefix.
    prefix: &str = search("prefix%3A%20{}%20", urlencoding::encode(prefix));

    /// Queries only bots that has this vote count.
    votes: usize = search("points%3A%20{votes}%20");

    /// Queries only bots that has this monthly vote count.
    monthly_votes: usize = search("monthlyPoints%3A%20{monthly_votes}%20");

    /// Queries only Top.gg certified bots or not.
    certified: bool = search("certifiedBot%3A%20{certified}%20");

    /// Queries only bots that has this Top.gg vanity URL.
    vanity: &str = search("vanity%3A%20{}%20", urlencoding::encode(vanity));
  }
}

impl<'a> IntoFuture for GetBots<'a> {
  type Output = crate::Result<Vec<Bot>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

  fn into_future(self) -> Self::IntoFuture {
    let mut query = self.query;

    if self.search.is_empty() {
      query.pop();
    } else {
      write!(&mut query, "search={}", self.search).unwrap();
    }

    if let Some(sort) = self.sort {
      write!(&mut query, "sort={sort}&").unwrap();
    }

    Box::pin(self.client.get_bots_inner(query))
  }
}
