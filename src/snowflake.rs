use crate::bot::Bot;
use core::{
  cmp::{Ordering, PartialEq, PartialOrd},
  fmt,
  ops::Deref,
};
use serde::de::{Deserialize, Deserializer, Error, Visitor};

struct SnowflakeVisitor;

impl<'de> Visitor<'de> for SnowflakeVisitor {
  type Value = Snowflake;

  #[inline(always)]
  fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("identifier")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(Snowflake(
      v.parse().map_err(|_| E::custom("invalid snowflake"))?,
    ))
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Snowflake(u64);

impl Deref for Snowflake {
  type Target = u64;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[allow(clippy::from_over_into)]
impl Into<u64> for Snowflake {
  #[inline(always)]
  fn into(self) -> u64 {
    self.0
  }
}

impl fmt::Display for Snowflake {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl PartialEq for Snowflake {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<S> PartialEq<S> for Snowflake
where
  S: SnowflakeLike,
{
  fn eq(&self, other: &S) -> bool {
    match other.as_snowflake() {
      Some(id) => id == self.0,
      None => false,
    }
  }
}

impl PartialOrd for Snowflake {
  #[inline(always)]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

impl<S> PartialOrd<S> for Snowflake
where
  S: SnowflakeLike,
{
  #[inline(always)]
  fn partial_cmp(&self, other: &S) -> Option<Ordering> {
    self.0.partial_cmp(&other.as_snowflake()?)
  }
}

impl<'de> Deserialize<'de> for Snowflake {
  #[inline(always)]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(SnowflakeVisitor)
  }
}

pub trait SnowflakeLike {
  fn as_snowflake(&self) -> Option<u64>;
}

impl SnowflakeLike for Bot {
  #[inline(always)]
  fn as_snowflake(&self) -> Option<u64> {
    Some(self.id.0)
  }
}

macro_rules! impl_snowflake_tryfrom(
  ($($t:ty),+) => {$(
    impl SnowflakeLike for $t {
      #[inline(always)]
      fn as_snowflake(&self) -> Option<u64> {
        (*self).try_into().ok()
      }
    }
  )+}
);

macro_rules! impl_snowflake_fromstr(
  ($($t:ty),+) => {$(
    impl SnowflakeLike for $t {
      #[inline(always)]
      fn as_snowflake(&self) -> Option<u64> {
        self.parse().ok()
      }
    }
  )+}
);

impl_snowflake_tryfrom!(u64, i128, u128, isize, usize);
impl_snowflake_fromstr!(str, String);
