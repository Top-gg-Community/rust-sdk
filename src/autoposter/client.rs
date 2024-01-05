use crate::{InnerClient, Stats};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait IntoClientSealed {
  type ArcInner: Send + Sync + 'static;

  fn get_arc(&self) -> Arc<Self::ArcInner>;
  async fn post_stats(arc: &Self::ArcInner, stats: &Stats);
}

/// A private trait that represents any datatype that can be interpreted as a [Top.gg API](https://docs.top.gg) Client.
///
/// This can either be a reference to an existing [`Client`][crate::Client] or a [`&str`][core::str] representing a [Top.gg API](https://docs.top.gg) token.
pub trait IntoClient: IntoClientSealed {}

#[async_trait::async_trait]
impl IntoClientSealed for str {
  type ArcInner = InnerClient;

  #[inline(always)]
  fn get_arc(&self) -> Arc<Self::ArcInner> {
    Arc::new(InnerClient::new(String::from(self)))
  }

  #[inline(always)]
  async fn post_stats(arc: &Self::ArcInner, stats: &Stats) {
    let _ = arc.post_stats(stats).await;
  }
}

impl IntoClient for str {}
