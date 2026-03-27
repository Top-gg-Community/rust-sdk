use serde::{Deserialize, Deserializer, de::Error};

pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer).and_then(|s| s.parse().map_err(D::Error::custom))
}

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    /// Any data type that can be interpreted as a Discord ID.
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub trait Snowflake {
      /// Converts this value to a [`u64`].
      fn as_snowflake(&self) -> u64;
    }

    macro_rules! impl_snowflake(
      ($(#[$attr:meta] )?$self:ident,$t:ty,$body:expr) => {
        $(#[$attr])?
        impl Snowflake for $t {
          fn as_snowflake(&$self) -> u64 {
            $body
          }
        }
      }
    );

    impl_snowflake!(self, u64, *self);

    macro_rules! impl_string(
      ($($t:ty),+) => {$(
        impl_snowflake!(self, $t, self.parse().expect("Invalid snowflake as it's not numeric."));
      )+}
    );

    impl_string!(&str, String);
  }
}
