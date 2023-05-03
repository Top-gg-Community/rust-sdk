use serde::{de::Error, Deserialize, Deserializer};

#[inline(always)]
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: Deserializer<'de>,
{
  Deserialize::deserialize(deserializer)
    .and_then(|s: &str| s.parse::<u64>().map_err(D::Error::custom))
}

#[inline(always)]
pub(crate) fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
  D: Deserializer<'de>,
{
  Deserialize::deserialize(deserializer)
    .map(|s: Vec<&str>| s.into_iter().filter_map(|next| next.parse().ok()).collect())
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
