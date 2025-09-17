use crate::InnerClient;
use std::sync::Arc;

pub trait AsClientSealed {
  fn as_client(&self) -> Arc<InnerClient>;
}

/// A private trait that represents any datatype that can be interpreted as a [Top.gg API](https://docs.top.gg) Client.
///
/// This can either be a reference to an existing [`Client`][crate::Client] or a [`&str`][std::str] representing a [Top.gg API](https://docs.top.gg) token.
pub trait AsClient: AsClientSealed {}

impl AsClientSealed for str {
  #[inline(always)]
  fn as_client(&self) -> Arc<InnerClient> {
    Arc::new(InnerClient::new(String::from(self)))
  }
}

impl AsClient for str {}
