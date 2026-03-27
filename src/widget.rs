use crate::{Platform, ProjectType, Snowflake};

/// Generates a large widget URL.
///
/// # Example
///
/// ```rust,no_run
/// let widget_url = topgg::widget::large(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn large<I>(platform: Platform, project_type: ProjectType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/large/{}/{}/{}",
    serde_variant::to_variant_name(&platform).unwrap(),
    serde_variant::to_variant_name(&project_type).unwrap(),
    id.as_snowflake()
  )
}

/// Generates a small widget URL for displaying votes.
///
/// # Example
///
/// ```rust,no_run
/// let widget_url = topgg::widget::votes(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn votes<I>(platform: Platform, project_type: ProjectType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/votes/{}/{}/{}",
    serde_variant::to_variant_name(&platform).unwrap(),
    serde_variant::to_variant_name(&project_type).unwrap(),
    id.as_snowflake()
  )
}

/// Generates a small widget URL for displaying a project's owner.
///
/// # Example
///
/// ```rust,no_run
/// let widget_url = topgg::widget::owner(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn owner<I>(platform: Platform, project_type: ProjectType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/owner/{}/{}/{}",
    serde_variant::to_variant_name(&platform).unwrap(),
    serde_variant::to_variant_name(&project_type).unwrap(),
    id.as_snowflake()
  )
}

/// Generates a small widget URL for displaying social stats.
///
/// # Example
///
/// ```rust,no_run
/// let widget_url = topgg::widget::social(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn social<I>(platform: Platform, project_type: ProjectType, id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!(
    "/widgets/small/social/{}/{}/{}",
    serde_variant::to_variant_name(&platform).unwrap(),
    serde_variant::to_variant_name(&project_type).unwrap(),
    id.as_snowflake()
  )
}
