use crate::{snowflake::Snowflake, util};
use chrono::{offset::Utc, DateTime};
use core::cmp::{min, PartialEq};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

/// A struct representing a Discord Bot listed on [top.gg](https://top.gg).
#[derive(Clone, Debug, Deserialize)]
pub struct Bot {
  /// The ID of this discord bot.
  pub id: Snowflake,

  /// The username of this discord bot.
  pub username: String,

  /// The discriminator of this discord bot.
  pub discriminator: String,

  /// The prefix of this discord bot.
  pub prefix: String,

  /// The short description of this discord bot.
  #[serde(rename = "shortdesc")]
  pub short_description: String,

  /// The long description of this discord bot. It can contain HTML and/or Markdown.
  #[serde(rename = "longdesc")]
  pub long_description: Option<String>,

  /// The tags of this discord bot.
  pub tags: Vec<String>,

  /// The website URL of this discord bot.
  pub website: Option<String>,

  /// The link to the github repo of this discord bot.
  pub github: Option<String>,

  /// A list of snowflakes of this discord bot's owners. The main owner is the first snowflake in the array.
  pub owners: Vec<Snowflake>,

  /// A list of snowflakes of the guilds featured on this discord bot's page.
  pub guilds: Vec<Snowflake>,

  /// The custom bot invite URL of this discord bot.
  pub invite: Option<String>,

  /// The URL for this discord bot's banner image.
  #[serde(rename = "bannerUrl")]
  pub banner_url: Option<String>,

  /// The date when this discord bot was approved on [top.gg](https://top.gg).
  pub date: DateTime<Utc>,

  /// Whether this discord bot is [top.gg](https://top.gg) certified or not.
  #[serde(rename = "certifiedBot")]
  pub is_certified: bool,

  /// A list of this discord bot's shards.
  pub shards: Option<Vec<u64>>,

  /// The amount of shards this discord bot has according to posted stats.
  pub shard_count: Option<u64>,

  /// The amount of upvotes this discord bot has.
  #[serde(rename = "points")]
  pub votes: u64,

  /// The amount of upvotes this discord bot has this month.
  #[serde(rename = "monthlyPoints")]
  pub monthly_votes: u64,

  avatar: Option<String>,
  support: Option<String>,
  vanity: Option<String>,
}

impl Bot {
  /// Retrieves the avatar URL of this bot.
  ///
  /// It's format will be either PNG or GIF if animated.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_bot_ever_made = client.get_bot(264811613708746752u64).await.unwrap();
  ///   
  ///   println!("{}", best_bot_ever_made.avatar());
  /// }
  /// ```
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, &self.discriminator, self.id.into())
  }

  /// Retrieves the URL of this discord bot's [top.gg](https://top.gg) page.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_bot_ever_made = client.get_bot(264811613708746752u64).await.unwrap();
  ///   
  ///   println!("{}", best_bot_ever_made.url());
  /// }
  /// ```
  #[inline(always)]
  pub fn url(&self) -> String {
    format!(
      "https://top.gg/bot/{}",
      self.vanity.as_deref().unwrap_or(&self.id.to_string())
    )
  }

  /// Retrieves the support server invite URL of this discord bot if available.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_bot_ever_made = client.get_bot(264811613708746752u64).await.unwrap();
  ///   
  ///   println!("{}", best_bot_ever_made.server());
  /// }
  /// ```
  pub fn server(&self) -> Option<String> {
    self
      .support
      .as_ref()
      .map(|support| format!("https://discord.com/invite/{support}"))
  }
}

impl PartialEq for Bot {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

#[derive(Deserialize)]
pub(crate) struct Bots {
  pub(crate) results: Vec<Bot>,
}

/// A struct representing a discord bot's statistics returned from the API.
#[derive(Clone, Debug, Deserialize)]
pub struct BotStats {
  /// The bot's server count if available.
  pub server_count: Option<u64>,

  /// The bot's server count per shard if available.
  pub shards: Option<Vec<u64>>,

  /// The bot's shard count if available.
  pub shard_count: Option<u64>,
}

/// A struct representing a discord bot's statistics [to be posted][`crate::Client::set_bot_stats`] to the API.
#[derive(Serialize)]
pub struct NewBotStats {
  pub(crate) server_count: u64,
  shards: Option<Vec<u64>>,
  shard_count: Option<u64>,
  shard_id: Option<u64>,
}

impl NewBotStats {
  /// Creates a new discord bot's statistics struct.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::NewBotStats;
  ///
  /// let _stats = NewBotStats::new();
  /// ```
  pub const fn new() -> Self {
    Self {
      server_count: 0,
      shards: None,
      shard_count: None,
      shard_id: None,
    }
  }

  /// Sets the server count for this struct - it must NOT be zero.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::NewBotStats;
  ///
  /// let _stats = NewBotStats::new()
  ///   .server_count(1234);
  /// ```
  pub fn server_count<S>(mut self, new_server_count: S) -> Self
  where
    S: Into<u64>,
  {
    self.server_count = new_server_count.into();
    self
  }

  /// Sets the shard count for this struct.
  ///
  /// # Panics
  ///
  /// Panics if a shards array is already provided through [`NewBotStats::shards`] and it's length doesn't match the `new_shard_count` argument.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::NewBotStats;
  ///
  /// let _stats = NewBotStats::new()
  ///   .server_count(1234)
  ///   .shard_count(69);
  /// ```
  pub fn shard_count<C>(mut self, new_shard_count: C) -> Self
  where
    C: Into<u64>,
  {
    let new_shard_count = new_shard_count.into();

    if let Some(ref new_shards) = self.shards {
      assert!(
        (new_shards.len() as u64) == new_shard_count,
        "new shard count doesn't match the shards array's length - please use .shards() instead"
      );
    }

    self.shard_count = Some(new_shard_count);
    self
  }

  /// Sets a list of server count per shard for this struct - and optionally an zero-index to the main shard posting the current stats.
  ///
  /// Please note that the server count will automatically be set as the sum of every server count in the shards array.
  ///
  /// # Panics
  ///
  /// Panics if `shard_index` is out of bounds from the shards array.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::NewBotStats;
  ///
  /// let _stats = NewBotStats::new()
  ///   .shards([123, 456, 789], None);
  /// ```
  ///
  /// Or:
  ///
  /// ```rust,no_run
  /// use topgg::NewBotStats;
  ///
  /// // the shard posting this has 123 servers in it.
  /// let _stats = NewBotStats::new()
  ///   .shards([123, 456, 789], Some(0));
  /// ```
  pub fn shards<A, I>(mut self, new_shards: A, shard_index: Option<I>) -> Self
  where
    A: IntoIterator,
    A::Item: Into<u64>,
    I: Into<u64>,
  {
    self.server_count = 0u64;
    let new_shards = new_shards.into_iter();
    let mut new_shards_list = Vec::with_capacity(new_shards.size_hint().0);

    for server_count in new_shards.map(|s| s.into()) {
      self.server_count += server_count;
      new_shards_list.push(server_count);
    }

    if let Some(shard_index) = shard_index {
      let shard_index = shard_index.into();

      assert!(
        shard_index < (new_shards_list.len() as u64),
        "shard index out of range"
      );

      self.shard_id = Some(shard_index);
    }

    self.shard_count = Some(new_shards_list.len() as _);
    self.shards = Some(new_shards_list);

    self
  }
}

/// Creates a new discord bot's statistics struct. (Same as [`NewBotStats::new`])
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use topgg::NewBotStats;
///
/// let _stats = NewBotStats::default();
/// ```
impl Default for NewBotStats {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Deserialize)]
pub(crate) struct IsWeekend {
  pub(crate) is_weekend: u8,
}

/// A struct for filtering the query in [`get_bots`][`crate::Client::get_bots`].
pub struct Filter(String);

impl Filter {
  /// Initiates a new empty filtering struct.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new();
  /// ```
  #[inline(always)]
  pub fn new() -> Self {
    Self(String::new())
  }

  /// Filters only discord bots that matches this username.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .username("shiro");
  /// ```
  pub fn username<U>(mut self, new_username: &U) -> Self
  where
    U: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("username: {} ", new_username.as_ref()));
    self
  }

  /// Filters only discord bots that matches this discriminator.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .discriminator("1536");
  /// ```
  pub fn discriminator<D>(mut self, new_discriminator: &D) -> Self
  where
    D: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("discriminator: {} ", new_discriminator.as_ref()));
    self
  }

  /// Filters only discord bots that matches this prefix.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .prefix("!");
  /// ```
  pub fn prefix<P>(mut self, new_prefix: &P) -> Self
  where
    P: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("prefix: {} ", new_prefix.as_ref()));
    self
  }

  /// Filters only discord bots that has this vote count.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .votes(1000);
  /// ```
  pub fn votes<V>(mut self, new_votes: V) -> Self
  where
    V: Into<u64>,
  {
    self.0.push_str(&format!("points: {} ", new_votes.into()));
    self
  }

  /// Filters only discord bots that has this monthly vote count.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .monthly_votes(100);
  /// ```
  pub fn monthly_votes<M>(mut self, new_monthly_votes: M) -> Self
  where
    M: Into<u64>,
  {
    self
      .0
      .push_str(&format!("monthlyPoints: {} ", new_monthly_votes.into()));
    self
  }

  /// Filters only [top.gg](https://top.gg) certified discord bots or not.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .certified(true);
  /// ```
  pub fn certified<C>(mut self, is_certified: C) -> Self
  where
    C: Into<bool>,
  {
    self
      .0
      .push_str(&format!("certifiedBot: {} ", is_certified.into()));
    self
  }

  /// Filters only discord bots that has this [top.gg](https://top.gg) vanity URL.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .vanity("mee6");
  /// ```
  pub fn vanity<V>(mut self, new_vanity: &V) -> Self
  where
    V: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("vanity: {} ", new_vanity.as_ref()));
    self
  }
}

/// Initiates a new empty filtering struct. (Same as [`Filter::new`])
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use topgg::Filter;
///
/// let _filter = Filter::default();
/// ```
impl Default for Filter {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

/// A struct for configuring the query in [`get_bots`][`crate::Client::get_bots`].
pub struct Query(String);

impl Query {
  /// Initiates a new empty querying struct.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Query;
  ///
  /// let _query = Query::new();
  /// ```
  #[inline(always)]
  pub fn new() -> Self {
    Self(String::from("?"))
  }

  /// Sets the maximum amount of bots to be queried - it can't exceed 500.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Query;
  ///
  /// let _query = Query::new()
  ///   .limit(250);
  /// ```
  pub fn limit<N>(mut self, new_limit: N) -> Self
  where
    N: Into<u16>,
  {
    self
      .0
      .push_str(&format!("limit={}&", min(new_limit.into(), 500)));
    self
  }

  /// Sets the amount of bots to be skipped during the query - it can't exceed 499.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Query;
  ///
  /// let _query = Query::new()
  ///   .limit(250)
  ///   .skip(100);
  /// ```
  pub fn skip<S>(mut self, skip_by: S) -> Self
  where
    S: Into<u16>,
  {
    self
      .0
      .push_str(&format!("offset={}&", min(skip_by.into(), 499)));
    self
  }

  /// Sets [`Filter`] instance to this query struct.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Filter, Query};
  ///
  /// let filter = Filter::new()
  ///   .username("shiro")
  ///   .certified(true);
  ///
  /// let _query = Query::new()
  ///   .limit(250)
  ///   .skip(100)
  ///   .filter(filter);
  /// ```
  pub fn filter(mut self, mut new_filter: Filter) -> Self {
    new_filter.0.pop();
    self
      .0
      .push_str(&format!("search={}&", encode(&new_filter.0)));
    self
  }
}

impl Default for Query {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

/// A trait that represents any data type that can be interpreted as a valid [top.gg](https://top.gg) discord bot query.
pub trait QueryLike {
  fn into_query_string(self) -> String;
}

impl QueryLike for Query {
  #[inline(always)]
  fn into_query_string(mut self) -> String {
    self.0.pop();
    self.0
  }
}

impl QueryLike for Filter {
  #[inline(always)]
  fn into_query_string(mut self) -> String {
    self.0.pop();
    format!("?search={}", encode(&self.0))
  }
}

impl<S> QueryLike for &S
where
  S: AsRef<str> + ?Sized,
{
  #[inline(always)]
  fn into_query_string(self) -> String {
    format!("?search=username%3A%20{}", encode(self.as_ref()))
  }
}
