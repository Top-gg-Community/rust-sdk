use crate::Error;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[inline(always)]
pub(crate) fn deserialize_optional_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(match <String as Deserialize>::deserialize(deserializer) {
    Ok(s) => {
      if s.is_empty() {
        None
      } else {
        Some(s)
      }
    }
    _ => None,
  })
}

#[inline(always)]
pub(crate) fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  T: Default + Deserialize<'de>,
  D: Deserializer<'de>,
{
  Option::deserialize(deserializer).map(|res| res.unwrap_or_default())
}

#[inline(always)]
pub(crate) fn get_creation_date(id: u64) -> DateTime<Utc> {
  Utc
    .timestamp_millis_opt(((id >> 22) + DISCORD_EPOCH) as _)
    .single()
    .unwrap()
}

#[inline(always)]
pub(crate) async fn parse_json<T>(response: Response) -> crate::Result<T>
where
  T: DeserializeOwned,
{
  if let Ok(bytes) = response.bytes().await {
    if let Ok(json) = serde_json::from_slice(&bytes) {
      return Ok(json);
    }
  }

  Err(Error::InternalServerError)
}

pub(crate) fn get_avatar(hash: &Option<String>, id: u64) -> String {
  match hash {
    Some(hash) => {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!("https://cdn.discordapp.com/avatars/{id}/{hash}.{ext}?size=1024")
    }
    _ => format!(
      "https://cdn.discordapp.com/embed/avatars/{}.png",
      (id >> 22) % 5
    ),
  }
}
