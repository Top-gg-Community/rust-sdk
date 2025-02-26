use crate::{snowflake, util, Client};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
  cmp::min,
  collections::HashMap,
  future::{Future, IntoFuture},
  pin::Pin,
};

util::debug_struct! {
  /// Represents a Discord bot listed on Top.gg.
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
      #[serde(rename = "username")]
      name: String,

      /// This bot's prefix.
      prefix: String,

      /// This bot's short description.
      #[serde(rename = "shortdesc")]
      short_description: String,

      /// This bot's long description. It can contain HTML and/or Markdown.
      #[serde(
        default,
        deserialize_with = "util::deserialize_optional_string",
        rename = "longdesc"
      )]
      long_description: Option<String>,

      /// This bot's tags.
      #[serde(deserialize_with = "util::deserialize_default")]
      tags: Vec<String>,

      /// This bot's website URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      website: Option<String>,

      /// This bot's GitHub repository URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      github: Option<String>,

      /// This bot's owners IDs.
      #[serde(deserialize_with = "snowflake::deserialize_vec")]
      owners: Vec<u64>,

      /// This bot's submission date.
      #[serde(rename = "date")]
      submitted_at: DateTime<Utc>,

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

      /// This bot's invite URL.
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      invite: Option<String>,

      /// This bot's posted server count.
      #[serde(default)]
      server_count: Option<usize>,
    }

    private {
      #[serde(default, deserialize_with = "util::deserialize_optional_string")]
      vanity: Option<String>,
    }

    getters(self) {
      /// This bot's creation date.
      #[must_use]
      #[inline(always)]
      created_at: DateTime<Utc> => {
        util::get_creation_date(self.id)
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

/// A struct for configuring the query in [`get_bots`][crate::Client::get_bots] before being sent to the API.
#[must_use]
pub struct BotQuery<'a> {
  client: &'a Client,
  query: HashMap<&'static str, String>,
  search: HashMap<&'static str, String>,
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
      search: HashMap::new(),
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
    /// Sets the maximum amount of bots to be queried. This cannot be more than 500.
    limit: u16 = query(limit, min(limit, 500).to_string());

    /// Sets the amount of bots to be skipped during the query. This cannot be more than 499.
    skip: u16 = query(offset, min(skip, 499).to_string());

    /// Queries only Discord bots that has this username.
    name: &str = search(username, urlencoding::encode(name).to_string());

    /// Queries only Discord bots that has this prefix.
    prefix: &str = search(prefix, urlencoding::encode(prefix).to_string());

    /// Queries only Discord bots that has this vote count.
    votes: usize = search(points, votes.to_string());

    /// Queries only Discord bots that has this monthly vote count.
    monthly_votes: usize = search(monthlyPoints, monthly_votes.to_string());

    /// Queries only Discord bots that has this Top.gg vanity URL.
    vanity: &str = search(vanity, urlencoding::encode(vanity).to_string());
  }
}

impl<'a> IntoFuture for BotQuery<'a> {
  type Output = crate::Result<Vec<Bot>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

  fn into_future(self) -> Self::IntoFuture {
    let mut path = String::from("/bots?");

    if let Some(sort) = self.sort {
      path.push_str(&format!("sort={sort}&"));
    }

    if !self.search.is_empty() {
      let mut search = String::new();

      for (key, value) in self.search {
        search.push_str(&format!("{key}%3A%20{value}%20"));
      }

      if !search.is_empty() {
        search.truncate(search.len() - 3);
      }

      path.push_str(&format!("search={search}&"));
    }

    for (key, value) in self.query {
      path.push_str(&format!("{key}={value}&"));
    }

    path.pop();

    Box::pin(self.client.get_bots_inner(path))
  }
}
