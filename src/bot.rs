use crate::{
  snowflake::{Snowflake, SnowflakeLike},
  util,
};
use chrono::{offset::Utc, DateTime};
use core::cmp::{min, PartialEq};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
  pub id: Snowflake,
  pub username: String,
  pub discriminator: String,
  pub prefix: String,
  #[serde(rename = "shortdesc")]
  pub short_description: String,
  #[serde(rename = "longdesc")]
  pub long_description: Option<String>,
  pub tags: Vec<String>,
  pub website: Option<String>,
  pub support: Option<String>,
  pub github: Option<String>,
  pub owners: Vec<Snowflake>,
  pub guilds: Vec<Snowflake>,
  pub invite: Option<String>,
  pub banner_url: Option<String>,
  pub date: DateTime<Utc>,
  #[serde(rename = "certifiedBot")]
  pub is_certified: bool,
  pub shards: Option<Vec<u64>>,
  #[serde(rename = "points")]
  pub votes: u64,
  #[serde(rename = "monthlyPoints")]
  pub monthly_votes: u64,
  avatar: Option<String>,
  vanity: Option<String>,
}

impl Bot {
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, &self.discriminator, self.id.into())
  }

  pub fn url(&self) -> String {
    format!(
      "https://top.gg/bot/{}",
      self.vanity.as_deref().unwrap_or(&self.id.to_string())
    )
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

#[derive(Clone, Debug, Deserialize)]
pub struct BotStats {
  pub server_count: Option<u64>,
  pub shards: Option<Vec<u64>>,
  pub shard_count: Option<u64>,
}

#[derive(Serialize)]
pub struct NewBotStats {
  pub(crate) server_count: u64,
  shards: Option<Vec<u64>>,
  shard_count: Option<u64>,
  shard_id: Option<u64>,
}

impl NewBotStats {
  pub const fn new() -> Self {
    Self {
      server_count: 0,
      shards: None,
      shard_count: None,
      shard_id: None,
    }
  }

  pub fn server_count<S>(mut self, new_server_count: S) -> Self
  where
    S: Into<u64>,
  {
    self.server_count = new_server_count.into();
    self
  }

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

pub struct Filter(String);

impl Filter {
  #[inline(always)]
  pub fn new() -> Self {
    Self(String::new())
  }

  pub fn username<U>(mut self, new_username: &U) -> Self
  where
    U: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("username: {} ", new_username.as_ref()));
    self
  }

  pub fn discriminator<D>(mut self, new_discriminator: &D) -> Self
  where
    D: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("discriminator: {} ", new_discriminator.as_ref()));
    self
  }

  pub fn prefix<P>(mut self, new_prefix: &P) -> Self
  where
    P: AsRef<str> + ?Sized,
  {
    self
      .0
      .push_str(&format!("prefix: {} ", new_prefix.as_ref()));
    self
  }

  pub fn id<I>(mut self, new_id: I) -> Self
  where
    I: SnowflakeLike,
  {
    self.0.push_str(&format!("id: {} ", new_id.as_snowflake()));
    self
  }

  pub fn votes<V>(mut self, new_votes: V) -> Self
  where
    V: Into<u64>,
  {
    self.0.push_str(&format!("points: {} ", new_votes.into()));
    self
  }

  pub fn monthly_votes<M>(mut self, new_monthly_votes: M) -> Self
  where
    M: Into<u64>,
  {
    self
      .0
      .push_str(&format!("monthlyPoints: {} ", new_monthly_votes.into()));
    self
  }

  pub fn certified<C>(mut self, is_certified: C) -> Self
  where
    C: Into<bool>,
  {
    self
      .0
      .push_str(&format!("certifiedBot: {} ", is_certified.into()));
    self
  }

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

impl Default for Filter {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

pub struct Query(String);

impl Query {
  #[inline(always)]
  pub fn new() -> Self {
    Self(String::from("?"))
  }

  pub fn limit<N>(mut self, new_limit: N) -> Self
  where
    N: Into<u16>,
  {
    self
      .0
      .push_str(&format!("limit={}&", min(new_limit.into(), 500)));
    self
  }

  pub fn skip<S>(mut self, skip_by: S) -> Self
  where
    S: Into<u16>,
  {
    self
      .0
      .push_str(&format!("offset={}&", min(skip_by.into(), 499)));
    self
  }

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

pub trait QueryLike {
  fn into_query_string(self) -> String;
}

impl QueryLike for Query {
  fn into_query_string(mut self) -> String {
    self.0.pop();
    self.0
  }
}

impl<S> QueryLike for &S
where
  S: AsRef<str> + ?Sized,
{
  fn into_query_string(self) -> String {
    format!("?search=username%3A%20{}", encode(self.as_ref()))
  }
}
