use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use core::{
  cmp::min,
  fmt::{self, Debug, Formatter},
};
use serde::{Deserialize, Deserializer, Serialize};

/// A struct representing a Discord Bot listed on [Top.gg](https://top.gg).
#[must_use]
#[derive(Clone, Deserialize)]
pub struct Bot {
  /// The ID of this Discord bot.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// The username of this Discord bot.
  pub username: String,

  #[deprecated(since = "1.1.0")]
  pub discriminator: String,

  /// The prefix of this Discord bot.
  pub prefix: String,

  /// The short description of this Discord bot.
  #[serde(rename = "shortdesc")]
  pub short_description: String,

  /// The long description of this Discord bot. It can contain HTML and/or Markdown.
  #[serde(
    default,
    deserialize_with = "util::deserialize_optional_string",
    rename = "longdesc"
  )]
  pub long_description: Option<String>,

  /// The tags of this Discord bot.
  #[serde(default, deserialize_with = "util::deserialize_default")]
  pub tags: Vec<String>,

  /// The website URL of this Discord bot.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub website: Option<String>,

  /// The link to this Discord bot's GitHub repository.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub github: Option<String>,

  /// A list of IDs of this Discord bot's owners. The main owner is the first ID in the array.
  #[serde(deserialize_with = "snowflake::deserialize_vec")]
  pub owners: Vec<u64>,

  /// A list of IDs of the guilds featured on this Discord bot's page.
  #[serde(default, deserialize_with = "snowflake::deserialize_vec")]
  pub guilds: Vec<u64>,

  /// The URL for this Discord bot's banner image.
  #[serde(
    default,
    deserialize_with = "util::deserialize_optional_string",
    rename = "bannerUrl"
  )]
  pub banner_url: Option<String>,

  /// The date when this Discord bot was approved on [Top.gg](https://top.gg).
  pub date: DateTime<Utc>,

  /// Whether this Discord bot is [Top.gg](https://top.gg) certified or not.
  #[serde(rename = "certifiedBot")]
  pub is_certified: bool,

  /// A list of this Discord bot's shards.
  #[serde(default, deserialize_with = "util::deserialize_default")]
  pub shards: Vec<u64>,

  /// The amount of upvotes this Discord bot has.
  #[serde(rename = "points")]
  pub votes: u64,

  /// The amount of upvotes this Discord bot has this month.
  #[serde(rename = "monthlyPoints")]
  pub monthly_votes: u64,

  /// The support server invite URL of this Discord bot.
  #[serde(default, deserialize_with = "deserialize_support_server")]
  pub support: Option<String>,

  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  avatar: Option<String>,

  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  invite: Option<String>,

  shard_count: Option<u64>,

  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  vanity: Option<String>,
}

#[inline(always)]
pub(crate) fn deserialize_support_server<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    unsafe { util::deserialize_optional_string(deserializer).unwrap_unchecked() }
      .map(|support| format!("https://discord.com/invite/{support}")),
  )
}

impl Bot {
  /// Retrieves the creation date of this bot.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///   
  ///   println!("{}", bot.created_at());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn created_at(&self) -> DateTime<Utc> {
    util::get_creation_date(self.id)
  }

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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///   
  ///   println!("{}", bot.avatar());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, self.id)
  }

  /// The invite URL of this Discord bot.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///   
  ///   println!("{}", bot.invite());
  /// }
  /// ```
  #[must_use]
  pub fn invite(&self) -> String {
    match self.invite.as_ref() {
      Some(inv) => inv.to_owned(),
      _ => format!(
        "https://discord.com/oauth2/authorize?scope=bot&client_id={}",
        self.id
      ),
    }
  }

  /// The amount of shards this Discord bot has according to posted stats.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///   
  ///   println!("{}", bot.shard_count());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn shard_count(&self) -> u64 {
    self.shard_count.unwrap_or(self.shards.len() as _)
  }

  /// Retrieves the URL of this Discord bot's [Top.gg](https://top.gg) page.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///   
  ///   println!("{}", bot.url());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn url(&self) -> String {
    format!(
      "https://top.gg/bot/{}",
      self.vanity.as_deref().unwrap_or(&self.id.to_string())
    )
  }
}

impl Debug for Bot {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    fmt
      .debug_struct("Bot")
      .field("id", &self.id)
      .field("username", &self.username)
      .field("prefix", &self.prefix)
      .field("short_description", &self.short_description)
      .field("long_description", &self.long_description)
      .field("tags", &self.tags)
      .field("website", &self.website)
      .field("github", &self.github)
      .field("owners", &self.owners)
      .field("guilds", &self.guilds)
      .field("banner_url", &self.banner_url)
      .field("date", &self.date)
      .field("is_certified", &self.is_certified)
      .field("shards", &self.shards)
      .field("votes", &self.votes)
      .field("monthly_votes", &self.monthly_votes)
      .field("support", &self.support)
      .field("avatar", &self.avatar())
      .field("invite", &self.invite())
      .field("shard_count", &self.shard_count())
      .field("url", &self.url())
      .finish()
  }
}

#[derive(Deserialize)]
pub(crate) struct Bots {
  pub(crate) results: Vec<Bot>,
}

/// A struct representing a Discord bot's statistics returned from the API.
#[must_use]
#[derive(Clone, Deserialize)]
pub struct Stats {
  /// The bot's server count per shard.
  #[serde(default, deserialize_with = "util::deserialize_default")]
  pub shards: Vec<u64>,

  shard_count: Option<u64>,
  server_count: Option<u64>,
}

impl Stats {
  /// The amount of shards this Discord bot has according to posted stats.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let stats = client.get_stats().await.unwrap();
  ///   
  ///   println!("{:?}", stats.shard_count());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn shard_count(&self) -> u64 {
    self.shard_count.unwrap_or(self.shards.len() as _)
  }

  /// The amount of servers this bot is in. `None` if such information is publicly unavailable.
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
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///   
  ///   let stats = client.get_stats().await.unwrap();
  ///   
  ///   println!("{:?}", stats.server_count());
  /// }
  /// ```
  #[must_use]
  pub fn server_count(&self) -> Option<u64> {
    self.server_count.or(if self.shards.is_empty() {
      None
    } else {
      Some(self.shards.iter().copied().sum())
    })
  }
}

impl Debug for Stats {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    fmt
      .debug_struct("Stats")
      .field("shards", &self.shards)
      .field("shard_count", &self.shard_count())
      .field("server_count", &self.server_count())
      .finish()
  }
}

/// A struct representing a Discord bot's statistics [to be posted][crate::Client::post_stats] to the API.
#[must_use]
#[derive(Clone, Serialize)]
pub struct NewStats {
  server_count: u64,
  shard_count: Option<u64>,
  shards: Option<Vec<u64>>,
  shard_id: Option<u64>,
}

impl NewStats {
  /// Creates a [`NewStats`] struct based on total server (and optionally, shard) count data.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::NewStats;
  ///
  /// let _stats = NewStats::count_based(12345, Some(10));
  /// ```
  pub const fn count_based(server_count: u64, shard_count: Option<u64>) -> Self {
    Self {
      server_count,
      shard_count,
      shards: None,
      shard_id: None,
    }
  }

  /// Creates a [`NewStats`] struct based on server count per shard and optionally the index (to the first array) of shard posting posting this data.
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
  /// use topgg::NewStats;
  ///
  /// // The shard posting this data has 456 servers.
  /// let _stats = NewStats::shards_based([123, 456, 789], Some(1));
  /// ```
  pub fn shards_based<A>(shards: A, shard_index: Option<u64>) -> Self
  where
    A: IntoIterator<Item = u64>,
  {
    let mut total_server_count = 0u64;
    let shards = shards.into_iter();
    let mut shards_list = Vec::with_capacity(shards.size_hint().0);

    for server_count in shards {
      total_server_count += server_count;
      shards_list.push(server_count);
    }

    if let Some(index) = shard_index {
      assert!(index < shards_list.len() as u64, "shard index out of range");
    }

    Self {
      server_count: total_server_count,
      shard_count: Some(shards_list.len() as _),
      shards: Some(shards_list),
      shard_id: shard_index,
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct IsWeekend {
  pub(crate) is_weekend: bool,
}

/// A struct for filtering the query in [`get_bots`][crate::Client::get_bots].
#[must_use]
#[derive(Clone)]
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

  /// Filters only Discord bots that matches this username.
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
    self.0.push_str(&format!(
      "username%3A%20{}%20",
      urlencoding::encode(new_username.as_ref())
    ));
    self
  }

  #[deprecated(since = "1.1.0")]
  pub fn discriminator<D>(mut self, new_discriminator: &D) -> Self
  where
    D: AsRef<str> + ?Sized,
  {
    self.0.push_str(&format!(
      "discriminator%3A%20{}%20",
      new_discriminator.as_ref()
    ));
    self
  }

  /// Filters only Discord bots that matches this prefix.
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
    self.0.push_str(&format!(
      "prefix%3A%20{}%20",
      urlencoding::encode(new_prefix.as_ref())
    ));
    self
  }

  /// Filters only Discord bots that has this vote count.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Filter;
  ///
  /// let _filter = Filter::new()
  ///   .votes(1000u64);
  /// ```
  pub fn votes<V>(mut self, new_votes: u64) -> Self {
    self.0.push_str(&format!("points%3A%20{new_votes}%20"));
    self
  }

  /// Filters only Discord bots that has this monthly vote count.
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
  pub fn monthly_votes(mut self, new_monthly_votes: u64) -> Self {
    self
      .0
      .push_str(&format!("monthlyPoints%3A%20{new_monthly_votes}%20"));
    self
  }

  /// Filters only [Top.gg](https://top.gg) certified Discord bots or not.
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
  pub fn certified(mut self, is_certified: bool) -> Self {
    self
      .0
      .push_str(&format!("certifiedBot%3A%20{is_certified}%20"));
    self
  }

  /// Filters only Discord bots that has this [Top.gg](https://top.gg) vanity URL.
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
    self.0.push_str(&format!(
      "vanity%3A%20{}%20",
      urlencoding::encode(new_vanity.as_ref())
    ));
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

/// A struct for configuring the query in [`get_bots`][crate::Client::get_bots].
#[must_use]
#[derive(Clone)]
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
  pub fn limit(mut self, new_limit: u16) -> Self {
    self.0.push_str(&format!("limit={}&", min(new_limit, 500)));
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
  pub fn skip<S>(mut self, skip_by: u16) -> Self {
    self.0.push_str(&format!("offset={}&", min(skip_by, 499)));
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
  pub fn filter(mut self, new_filter: Filter) -> Self {
    self
      .0
      .push_str(&format!("search={}&", new_filter.into_query_string()));
    self
  }
}

impl Default for Query {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

/// A trait that represents any data type that can be interpreted as a valid [Top.gg](https://top.gg) Discord bot query.
pub trait QueryLike {
  #[doc(hidden)]
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
    if self.0.is_empty() {
      String::new()
    } else {
      self.0.truncate(self.0.len() - 3);

      format!("?search={}", self.0)
    }
  }
}

impl<S> QueryLike for &S
where
  S: AsRef<str> + ?Sized,
{
  #[inline(always)]
  fn into_query_string(self) -> String {
    format!(
      "?search=username%3A%20{}",
      urlencoding::encode(self.as_ref())
    )
  }
}
