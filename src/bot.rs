use crate::{snowflake, util, Client};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
  cmp::min,
  collections::HashMap,
  fmt::Write,
  future::{Future, IntoFuture},
  pin::Pin,
};

/// A Discord bot's reviews on Top.gg.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct BotReviews {
  /// This bot's average review score out of 5.
  #[serde(rename = "averageScore")]
  pub score: f64,

  /// This bot's review count.
  pub count: usize,
}

/// A Discord bot listed on Top.gg.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Bot {
  /// This bot's Discord ID.
  #[serde(rename = "clientid", deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// This bot's Top.gg ID.
  #[serde(rename = "id", deserialize_with = "snowflake::deserialize")]
  pub topgg_id: u64,

  /// This bot's username.
  #[serde(rename = "username")]
  pub name: String,

  /// This bot's prefix.
  pub prefix: String,

  /// This bot's short description.
  #[serde(rename = "shortdesc")]
  pub short_description: String,

  /// This bot's HTML/Markdown long description.
  #[serde(
    default,
    deserialize_with = "util::deserialize_optional_string",
    rename = "longdesc"
  )]
  pub long_description: Option<String>,

  /// This bot's tags.
  #[serde(deserialize_with = "util::deserialize_default")]
  pub tags: Vec<String>,

  /// This bot's website URL.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub website: Option<String>,

  /// This bot's GitHub repository URL.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub github: Option<String>,

  /// This bot's owner IDs.
  #[serde(deserialize_with = "snowflake::deserialize_vec")]
  pub owners: Vec<u64>,

  /// This bot's submission date.
  #[serde(rename = "date")]
  pub submitted_at: DateTime<Utc>,

  /// The amount of votes this bot has.
  #[serde(rename = "points")]
  pub votes: usize,

  /// The amount of votes this bot has this month.
  #[serde(rename = "monthlyPoints")]
  pub monthly_votes: usize,

  /// This bot's support URL.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub support: Option<String>,

  /// This bot's avatar URL.
  pub avatar: String,

  /// This bot's invite URL.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub invite: Option<String>,

  /// This bot's Top.gg vanity code.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub vanity: Option<String>,

  /// This bot's posted server count.
  #[serde(default)]
  pub server_count: Option<usize>,

  /// This bot's reviews.
  #[serde(rename = "reviews")]
  pub review: BotReviews,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Stats {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) server_count: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct Bots {
  pub(crate) results: Vec<Bot>,
}

#[derive(Deserialize)]
pub(crate) struct IsWeekend {
  pub(crate) is_weekend: bool,
}

/// Query for [`Client::get_bots`].
#[must_use]
pub struct BotQuery<'a> {
  client: &'a Client,
  query: HashMap<&'static str, String>,
  sort: Option<&'static str>,
}

macro_rules! get_bots_method {
  ($(
    $(#[doc = $doc:literal])*
    $lib_name:ident: $lib_type:ty = $property:ident($api_name:ident, $lib_value:expr);
  )*) => {$(
    $(#[doc = $doc])*
    pub fn $lib_name(mut self, $lib_name: $lib_type) -> Self {
      self.$property.insert(stringify!($api_name), $lib_value);
      self
    }
  )*};
}

macro_rules! get_bots_sort {
  ($(
    $(#[doc = $doc:literal])*
    $func_name:ident: $api_name:ident,
  )*) => {$(
    $(#[doc = $doc])*
    pub fn $func_name(mut self) -> Self {
      self.sort.replace(stringify!($api_name));
      self
    }
  )*};
}

impl<'a> BotQuery<'a> {
  #[inline(always)]
  pub(crate) fn new(client: &'a Client) -> Self {
    Self {
      client,
      query: HashMap::new(),
      sort: None,
    }
  }

  get_bots_sort! {
    /// Sorts results based on each bot's ID.
    sort_by_id: id,

    /// Sorts results based on each bot's submission date.
    sort_by_submission_date: date,

    /// Sorts results based on each bot's monthly vote count.
    sort_by_monthly_votes: monthlyPoints,
  }

  get_bots_method! {
    /// Sets the maximum amount of bots to be returned.
    limit: u16 = query(limit, min(limit, 500).to_string());

    /// Sets the amount of bots to be skipped.
    skip: u16 = query(offset, min(skip, 499).to_string());
  }
}

impl<'a> IntoFuture for BotQuery<'a> {
  type Output = crate::Result<Vec<Bot>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

  fn into_future(self) -> Self::IntoFuture {
    let mut path = String::from("/bots?");

    if let Some(sort) = self.sort {
      write!(&mut path, "sort={sort}&").unwrap();
    }

    for (key, value) in self.query {
      write!(&mut path, "{key}={value}&").unwrap();
    }

    path.pop();

    Box::pin(self.client.get_bots_inner(path))
  }
}
