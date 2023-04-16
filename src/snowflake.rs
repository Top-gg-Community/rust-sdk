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

  #[inline(always)]
  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(Snowflake(unsafe { v.parse().unwrap_unchecked() }))
  }
}

/// Represents a discord snowflake/ID.
#[derive(Copy, Clone, Debug)]
pub struct Snowflake(u64);

impl Deref for Snowflake {
  type Target = u64;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// Coerces this snowflake to a [`u64`].
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
  #[inline(always)]
  fn eq(&self, other: &S) -> bool {
    other.as_snowflake() == self.0
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
    self.0.partial_cmp(&other.as_snowflake())
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

/// A trait that represents any data type that can be interpreted as a snowflake.
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
