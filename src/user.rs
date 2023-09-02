use crate::{snowflake, util};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug, Formatter};
use serde::{Deserialize, Deserializer};

/// A struct representing a user's social links.
#[derive(Clone, Debug, Deserialize)]
pub struct Socials {
  /// A URL to this user's GitHub account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub github: Option<String>,

  /// A URL to this user's Instagram account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub instagram: Option<String>,

  /// A URL to this user's Reddit account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub reddit: Option<String>,

  /// A URL to this user's Twitter account.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub twitter: Option<String>,

  /// A URL to this user's YouTube channel.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub youtube: Option<String>,
}

/// A struct representing a user logged into [Top.gg](https://top.gg).
#[must_use]
#[derive(Clone, Deserialize)]
pub struct User {
  /// The Discord ID of this user.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// The username of this user.
  pub username: String,

  #[serde(deserialize_with = "deserialize_zero")]
  #[deprecated(
    since = "1.1.0",
    note = "deprecated in favor of discord's migration from using discriminators in usernames to using display names."
  )]
  pub discriminator: String,

  /// The user's bio.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub bio: Option<String>,

  /// A URL to this user's profile banner image.
  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  pub banner: Option<String>,

  /// A struct of this user's social links.
  #[serde(rename = "social")]
  pub socials: Option<Socials>,

  /// Whether this user is a [Top.gg](https://top.gg) supporter or not.
  #[serde(rename = "supporter")]
  pub is_supporter: bool,

  /// Whether this user is a [Top.gg](https://top.gg) certified developer or not.
  #[serde(rename = "certifiedDev")]
  pub is_certified_dev: bool,

  /// Whether this user is a [Top.gg](https://top.gg) moderator or not.
  #[serde(rename = "mod")]
  pub is_moderator: bool,

  /// Whether this user is a [Top.gg](https://top.gg) website moderator or not.
  #[serde(rename = "webMod")]
  pub is_web_moderator: bool,

  /// Whether this user is a [Top.gg](https://top.gg) website administrator or not.
  #[serde(rename = "admin")]
  pub is_admin: bool,

  #[serde(default, deserialize_with = "util::deserialize_optional_string")]
  avatar: Option<String>,
}

#[inline(always)]
pub(crate) fn deserialize_zero<'de, D>(_deserializer: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(String::from('0'))
}

impl User {
  /// Retrieves the creation date of this user.
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
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   
  ///   let user = client.get_user(661200758510977084).await.unwrap();
  ///   
  ///   println!("{}", user.created_at());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn created_at(&self) -> DateTime<Utc> {
    util::get_creation_date(self.id)
  }

  /// Retrieves the Discord avatar URL of this user.
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
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   
  ///   let user = client.get_user(661200758510977084).await.unwrap();
  ///   
  ///   println!("{}", user.avatar());
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, self.id)
  }
}

impl Debug for User {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    fmt
      .debug_struct("User")
      .field("id", &self.id)
      .field("username", &self.username)
      .field("bio", &self.bio)
      .field("banner", &self.banner)
      .field("socials", &self.socials)
      .field("is_supporter", &self.is_supporter)
      .field("is_certified_dev", &self.is_certified_dev)
      .field("is_moderator", &self.is_moderator)
      .field("is_web_moderator", &self.is_web_moderator)
      .field("is_admin", &self.is_admin)
      .field("created_at", &self.created_at())
      .field("avatar", &self.avatar())
      .finish()
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

/// A struct representing a user who has voted on a Discord bot listed on [Top.gg](https://top.gg). (See [`Client::get_voters`][crate::Client::get_voters])
#[must_use]
#[derive(Clone, Deserialize)]
pub struct Voter {
  /// The Discord ID of this user.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// The username of this user.
  pub username: String,

  avatar: Option<String>,
}

impl Voter {
  /// Retrieves the creation date of this user.
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
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   
  ///   for voter in client.get_voters().await.unwrap() {
  ///     println!("{}", voter.created_at());
  ///   }
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn created_at(&self) -> DateTime<Utc> {
    util::get_creation_date(self.id)
  }

  /// Retrieves the Discord avatar URL of this user.
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
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   
  ///   for voter in client.get_voters().await.unwrap() {
  ///     println!("{}", voter.avatar());
  ///   }
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, self.id)
  }
}

impl Debug for Voter {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    fmt
      .debug_struct("Voter")
      .field("id", &self.id)
      .field("username", &self.username)
      .field("created_at", &self.created_at())
      .field("avatar", &self.avatar())
      .finish()
  }
}
