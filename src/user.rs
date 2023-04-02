use crate::{snowflake::Snowflake, util};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Socials {
  pub github: Option<String>,
  pub instagram: Option<String>,
  pub reddit: Option<String>,
  pub twitter: Option<String>,
  pub youtube: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
  pub id: Snowflake,
  pub username: String,
  pub discriminator: String,
  pub bio: Option<String>,
  pub banner: Option<String>,
  #[serde(rename = "social")]
  pub socials: Option<Socials>,
  #[serde(rename = "supporter")]
  pub is_supporter: bool,
  #[serde(rename = "isCertifiedDev")]
  pub is_certified_dev: bool,
  #[serde(rename = "mod")]
  pub is_moderator: bool,
  #[serde(rename = "webMod")]
  pub is_web_moderator: bool,
  #[serde(rename = "admin")]
  pub is_admin: bool,
  avatar: Option<String>,
}

impl User {
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, &self.discriminator, self.id.into())
  }
}

#[derive(Deserialize)]
pub(crate) struct Voted {
  pub(crate) voted: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Voter {
  pub id: Snowflake,
  pub username: String,
  pub discriminator: String,
  avatar: Option<String>,
}

impl Voter {
  #[inline(always)]
  pub fn avatar(&self) -> String {
    util::get_avatar(&self.avatar, &self.discriminator, self.id.into())
  }
}
