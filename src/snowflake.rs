use serde::{de::Error, Deserialize, Deserializer};

#[inline(always)]
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer).and_then(|s| s.parse().map_err(D::Error::custom))
}

#[inline(always)]
#[cfg(feature = "api")]
pub(crate) fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
  D: Deserializer<'de>,
{
  Deserialize::deserialize(deserializer)
    .map(|s: Vec<String>| s.into_iter().filter_map(|next| next.parse().ok()).collect())
}

mod private {
  pub trait Sealed {}
}

/// A trait that represents any data type that can be interpreted as a snowflake/ID.
pub trait SnowflakeLike: private::Sealed {
  #[doc(hidden)]
  fn as_snowflake(&self) -> u64;
}

macro_rules! impl_snowflake_as(
  ($($t:ty),+) => {$(
    impl private::Sealed for $t {}

    impl SnowflakeLike for $t {
      #[inline(always)]
      fn as_snowflake(&self) -> u64 {
        *self as _
      }
    }
  )+}
);

impl<S> private::Sealed for &S where S: AsRef<str> + ?Sized {}

impl<S> SnowflakeLike for &S
where
  S: AsRef<str> + ?Sized,
{
  #[inline(always)]
  fn as_snowflake(&self) -> u64 {
    (*self).as_ref().parse().unwrap()
  }
}

impl_snowflake_as!(i64, u64, i128, u128, isize, usize);

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    use crate::{
      bot::Bot,
      user::{User, Voter},
    };

    macro_rules! impl_snowflake_idstruct(
      ($($t:ty),+) => {$(
        impl private::Sealed for $t {}
        impl private::Sealed for &$t {}

        impl SnowflakeLike for $t {
          #[inline(always)]
          fn as_snowflake(&self) -> u64 {
            self.id
          }
        }

        impl SnowflakeLike for &$t {
          #[inline(always)]
          fn as_snowflake(&self) -> u64 {
            (*self).id
          }
        }
      )+}
    );

    impl_snowflake_idstruct!(Bot, User, Voter);
  }
}
