use crate::Snowflake;

/// Widget type.
#[non_exhaustive]
pub enum WidgetType {
  DiscordBot,
  DiscordServer,
}

impl WidgetType {
  const fn as_path(&self) -> &'static str {
    match self {
      Self::DiscordBot => "discord/bot",
      Self::DiscordServer => "discord/server",
    }
  }
}

/// Generates a large widget URL.
#[inline(always)]
pub fn large<I>(ty: WidgetType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!("/widgets/large/{}/{}", ty.as_path(), id.as_snowflake())
}

/// Generates a small widget URL for displaying votes.
#[inline(always)]
pub fn votes<I>(ty: WidgetType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/votes/{}/{}",
    ty.as_path(),
    id.as_snowflake()
  )
}

/// Generates a small widget URL for displaying an entity's owner.
#[inline(always)]
pub fn owner<I>(ty: WidgetType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/owner/{}/{}",
    ty.as_path(),
    id.as_snowflake()
  )
}

/// Generates a small widget URL for displaying social stats.
#[inline(always)]
pub fn social<I>(ty: WidgetType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/social/{}/{}",
    ty.as_path(),
    id.as_snowflake()
  )
}
