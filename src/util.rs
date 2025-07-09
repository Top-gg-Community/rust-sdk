use crate::{snowflake, Error};
use base64::Engine;
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

#[inline(always)]
pub(crate) fn deserialize_optional_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    String::deserialize(deserializer)
      .ok()
      .filter(|s| !s.is_empty()),
  )
}

#[inline(always)]
pub(crate) fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  T: Default + Deserialize<'de>,
  D: Deserializer<'de>,
{
  Option::deserialize(deserializer).map(Option::unwrap_or_default)
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

#[derive(Deserialize)]
struct TokenStructure {
  #[serde(deserialize_with = "snowflake::deserialize")]
  id: u64,
}

pub(crate) fn id_from_token(token: &str) -> u64 {
  if let Some(base64_section) = token.split('.').nth(1) {
    if let Ok(decoded_base64) =
      base64::engine::general_purpose::STANDARD_NO_PAD.decode(base64_section)
    {
      if let Ok(token_structure) = serde_json::from_slice::<TokenStructure>(&decoded_base64) {
        return token_structure.id;
      }
    }
  }

  panic!("Got a malformed API token.");
}
