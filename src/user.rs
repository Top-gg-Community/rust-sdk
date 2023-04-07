use crate::{snowflake::Snowflake, util};
use serde::Deserialize;

/// A struct representing a user's social links if available.
#[derive(Clone, Debug, Deserialize)]
pub struct Socials {
  /// A URL to this user's GitHub account if available.
  pub github: Option<String>,

  /// A URL to this user's Instagram account if available.
  pub instagram: Option<String>,

  /// A URL to this user's Reddit account if available.
  pub reddit: Option<String>,

  /// A URL to this user's Twitter account if available.
  pub twitter: Option<String>,

  /// A URL to this user's YouTube channel if available.
  pub youtube: Option<String>,
}

/// A struct representing a user logged into [top.gg](https://top.gg).
#[derive(Clone, Debug, Deserialize)]
pub struct User {
  /// The Discord ID of this user.
  pub id: Snowflake,

  /// The username of this user.
  pub username: String,

  /// The Discord discriminator of this user.
  pub discriminator: String,

  /// The user's bio if available.
  pub bio: Option<String>,

  /// A URL to this user's profile banner image if available.
  pub banner: Option<String>,

  /// A struct of this user's social links if available.
  #[serde(rename = "social")]
  pub socials: Option<Socials>,

  /// Whether this user is a [top.gg](https://top.gg) supporter or not.
  #[serde(rename = "supporter")]
  pub is_supporter: bool,

  /// Whether this user is a [top.gg](https://top.gg) certified developer or not.
  #[serde(rename = "isCertifiedDev")]
  pub is_certified_dev: bool,

  /// Whether this user is a [top.gg](https://top.gg) moderator or not.
  #[serde(rename = "mod")]
  pub is_moderator: bool,

  /// Whether this user is a [top.gg](https://top.gg) website moderator or not.
  #[serde(rename = "webMod")]
  pub is_web_moderator: bool,

  /// Whether this user is a [top.gg](https://top.gg) website administrator or not.
  #[serde(rename = "admin")]
  pub is_admin: bool,

  avatar: Option<String>,
}

impl User {
  /// Retrieves the discord avatar URL of this user.
  ///
  /// It's format will be either PNG or GIF if animated.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use std::env;
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  ///   let client = Client::new(token);
  ///   
  ///   let user = client.get_user(661200758510977084u64).await.unwrap();
  ///   
  ///   println!("{}", user.avatar());
  /// }
  /// ```
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, &self.discriminator, self.id.into())
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

/// A struct representing a user who has voted on a discord bot listed on [top.gg](https://top.gg). (See [crate::Client::get_bot_voters`])
#[derive(Clone, Debug, Deserialize)]
pub struct Voter {
  /// The Discord ID of this user.
  pub id: Snowflake,

  /// The username of this user.
  pub username: String,

  avatar: Option<String>,
}

impl Voter {
  /// Retrieves the discord avatar URL of this user if available.
  ///
  /// It's format will be either PNG or GIF if animated.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use std::env;
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  ///   let client = Client::new(token);
  ///   let my_bot_id = 123456789u64;
  ///   
  ///   for voter in client.get_bot_voters(my_bot_id).await.unwrap() {
  ///     println!("{}", voter.avatar().unwrap_or(String::from("No avatar :(")));
  ///   }
  /// }
  /// ```
  #[must_use]
  pub fn avatar(&self) -> Option<String> {
    self.avatar.as_ref().map(|hash| {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!(
        "https://cdn.discordapp.com/avatars/{}/{hash}.{ext}?size=1024",
        self.id
      )
    })
  }
}
