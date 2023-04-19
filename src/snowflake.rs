use serde::de::{Deserialize, Deserializer, Error};

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: Deserializer<'de>,
{
  let s: &str = Deserialize::deserialize(deserializer)?;

  s.parse::<u64>().map_err(D::Error::custom)
}

pub(crate) fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
  D: Deserializer<'de>,
{
  let s: Vec<&str> = Deserialize::deserialize(deserializer)?;
  let out = Vec::with_capacity(s.len());

  Ok(s.into_iter().fold(out, |mut acc, next| {
    if let Ok(next) = next.parse::<u64>() {
      acc.push(next);
    }

    acc
  }))
}

/// A trait that represents any data type that can be interpreted as a snowflake/ID.
pub trait SnowflakeLike {
  #[doc(hidden)]
  fn as_snowflake(&self) -> u64;
}

macro_rules! impl_snowflake_tryfrom(
  ($($t:ty),+) => {$(
    impl SnowflakeLike for $t {
      #[inline(always)]
      fn as_snowflake(&self) -> u64 {
        (*self).try_into().unwrap()
      }
    }
  )+}
);

macro_rules! impl_snowflake_fromstr(
  ($($t:ty),+) => {$(
    impl SnowflakeLike for $t {
      #[inline(always)]
      fn as_snowflake(&self) -> u64 {
        self.parse().expect("invalid snowflake")
      }
    }
  )+}
);

impl_snowflake_tryfrom!(u64, i128, u128, isize, usize);
impl_snowflake_fromstr!(str, String);
