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

pub trait SnowflakeSealed {
  fn as_snowflake(&self) -> u64;
}

/// A private trait that represents any datatype that can be interpreted as a Discord snowflake/ID.
pub trait Snowflake: SnowflakeSealed {}

macro_rules! impl_snowflake(
  ($self:ident,$t:ty,$body:expr) => {
    impl Snowflake for $t {}

    impl SnowflakeSealed for $t {
      #[inline(always)]
      fn as_snowflake(&$self) -> u64 {
        $body
      }
    }
  }
);

impl_snowflake!(self, u64, *self);

macro_rules! impl_string(
  ($($t:ty),+) => {$(
    impl_snowflake!(self, $t, (*self).parse().expect("invalid snowflake as it's not numeric"));
  )+}
);

impl_string!(&str, String);

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    macro_rules! impl_topgg_idstruct(
      ($($t:ty),+) => {$(
        impl_snowflake!(self, &$t, (*self).id);
      )+}
    );

    impl_topgg_idstruct!(
      crate::bot::Bot,
      crate::user::User,
      crate::user::Voter
    );
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "serenity")] {
    impl_snowflake!(
      self,
      &serenity::model::guild::Member,
      (*self).user.id.get()
    );

    impl_snowflake!(
      self,
      &serenity::model::guild::PartialMember,
      (*self).user.as_ref().expect("user property in PartialMember is None").id.get()
    );

    macro_rules! impl_serenity_id(
      ($($t:ty),+) => {$(
        impl_snowflake!(self, $t, (*self).get());
      )+}
    );

    impl_serenity_id!(
      serenity::model::id::GenericId,
      serenity::model::id::UserId
    );

    macro_rules! impl_serenity_idstruct(
      ($($t:ty),+) => {$(
        impl_snowflake!(self, &$t, (*self).id.get());
      )+}
    );

    impl_serenity_idstruct!(
      serenity::model::gateway::PresenceUser,
      serenity::model::user::CurrentUser,
      serenity::model::user::User
    );
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "serenity-cached")] {
    use core::ops::Deref;

    macro_rules! impl_serenity_cacheref(
      ($($t:ty),+) => {$(
        impl Snowflake for $t {}

        impl SnowflakeSealed for $t {
          #[inline(always)]
          fn as_snowflake(&self) -> u64 {
            SnowflakeSealed::as_snowflake(&self.deref())
          }
        }
      )+}
    );

    impl_serenity_cacheref!(
      serenity::cache::UserRef<'_>,
      serenity::cache::MemberRef<'_>,
      serenity::cache::CurrentUserRef<'_>
    );
  }
}
