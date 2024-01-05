use crate::{util, Stats};
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

pub struct MakeshiftClient {
  client: reqwest::Client,
  token: String,
}

#[async_trait::async_trait]
impl IntoClientSealed for str {
  type ArcInner = MakeshiftClient;

  #[inline(always)]
  fn get_arc(&self) -> Arc<Self::ArcInner> {
    let mut token = String::from(self);
    token.insert_str(0, "Bearer ");

    Arc::new(Self::ArcInner {
      client: reqwest::Client::new(),
      token,
    })
  }

  #[inline(always)]
  async fn post_stats(arc: &Self::ArcInner, stats: &Stats) {
    let _ = util::post_stats(&arc.client, &arc.token, stats).await;
  }
}

impl IntoClient for str {}
