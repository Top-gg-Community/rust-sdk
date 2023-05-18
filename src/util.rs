use chrono::{naive::NaiveDateTime, DateTime, Utc};
use serde::{Deserialize, Deserializer};

const DISCORD_EPOCH: u64 = 1420070400000;

#[inline(always)]
pub(crate) fn deserialize_optional_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    Deserialize::deserialize(deserializer).ok().and_then(
      |s: String| {
        if s.is_empty() {
          None
        } else {
          Some(s)
        }
      },
    ),
  )
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
  DateTime::from_utc(
    unsafe {
      NaiveDateTime::from_timestamp_millis(((id >> 22) + DISCORD_EPOCH) as _).unwrap_unchecked()
    },
    Utc,
  )
}

pub(crate) fn get_avatar(hash: &Option<String>, id: u64) -> String {
  match hash {
    Some(hash) => {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!("https://cdn.discordapp.com/avatars/{id}/{hash}.{ext}?size=1024")
    },
    _ => format!(
      "https://cdn.discordapp.com/embed/avatars/{}.png",
      (id >> 22) % 5
    ),
  }
}
