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

/// A trait that represents any datatype that can be interpreted as a Discord snowflake/ID.
pub trait Snowflake {
  /// The method that converts this value to a [`u64`].
  fn as_snowflake(&self) -> u64;
}

macro_rules! impl_snowflake(
  ($(#[$attr:meta] )?$self:ident,$t:ty,$body:expr) => {
    $(#[$attr])?
    impl Snowflake for $t {
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
      #[cfg_attr(docsrs, doc(cfg(feature = "serenity")))] self,
      &serenity::model::guild::Member,
      (*self).user.id.get()
    );

    impl_snowflake!(
      #[cfg_attr(docsrs, doc(cfg(feature = "serenity")))] self,
      &serenity::model::guild::PartialMember,
      (*self).user.as_ref().expect("user property in PartialMember is None").id.get()
    );

    macro_rules! impl_serenity_id(
      ($($t:ty),+) => {$(
        impl_snowflake!(#[cfg_attr(docsrs, doc(cfg(feature = "serenity")))] self, $t, (*self).get());
      )+}
    );

    impl_serenity_id!(
      serenity::model::id::GenericId,
      serenity::model::id::UserId
    );

    macro_rules! impl_serenity_idstruct(
      ($($t:ty),+) => {$(
        impl_snowflake!(#[cfg_attr(docsrs, doc(cfg(feature = "serenity")))] self, &$t, (*self).id.get());
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
    use std::ops::Deref;

    macro_rules! impl_serenity_cacheref(
      ($($t:ty),+) => {$(
        impl_snowflake!(#[cfg_attr(docsrs, doc(cfg(feature = "serenity-cached")))] self, $t, Snowflake::as_snowflake(&self.deref()));
      )+}
    );

    impl_serenity_cacheref!(
      serenity::cache::UserRef<'_>,
      serenity::cache::MemberRef<'_>,
      serenity::cache::CurrentUserRef<'_>
    );
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "twilight")] {
    #[cfg_attr(docsrs, doc(cfg(feature = "twilight")))]
    impl<I> Snowflake for twilight_model::id::Id<I> {
      #[inline(always)]
      fn as_snowflake(&self) -> u64 {
        self.get()
      }
    }

    impl_snowflake!(#[cfg_attr(docsrs, doc(cfg(feature = "twilight")))] self, twilight_model::gateway::presence::UserOrId, match self {
      twilight_model::gateway::presence::UserOrId::User(user) => user.id.get(),
      twilight_model::gateway::presence::UserOrId::UserId { id } => id.get(),
    });

    macro_rules! impl_twilight_idstruct(
      ($($t:ty),+) => {$(
        impl_snowflake!(#[cfg_attr(docsrs, doc(cfg(feature = "twilight")))] self, &$t, (*self).id.get());
      )+}
    );

    impl_twilight_idstruct!(
      twilight_model::user::CurrentUser,
      twilight_model::user::User,
      twilight_model::user::UserProfile,
      twilight_model::gateway::payload::incoming::invite_create::PartialUser
    );
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "twilight-cached")] {
    impl_snowflake!(
      #[cfg_attr(docsrs, doc(cfg(feature = "twilight-cached")))] self,
      &twilight_cache_inmemory::model::CachedMember,
      (*self).user_id().get()
    );
  }
}
