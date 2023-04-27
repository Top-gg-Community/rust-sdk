use serde::de::{Deserialize, Deserializer};

#[inline(always)]
pub(crate) fn deserialize_optional_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(
    Deserialize::deserialize(deserializer)
      .ok()
      .and_then(|s: &str| {
        if s.is_empty() {
          None
        } else {
          Some(s.to_owned())
        }
      }),
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

pub(crate) fn get_avatar(hash: &Option<String>, discriminator: &str, id: u64) -> String {
  match hash {
    Some(hash) => {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!("https://cdn.discordapp.com/avatars/{id}/{hash}.{ext}?size=1024")
    }

    None => format!("https://cdn.discordapp.com/embed/avatars/{}.png", unsafe {
      discriminator.parse::<u16>().unwrap_unchecked() % 5
    }),
  }
}
