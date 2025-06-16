use crate::Snowflake;

/// Generates a large widget URL.
#[inline(always)]
pub fn large<I>(id: I) -> String
where
  I: Snowflake,
{
  crate::client::api!("/widgets/large/{}", id.as_snowflake())
}
