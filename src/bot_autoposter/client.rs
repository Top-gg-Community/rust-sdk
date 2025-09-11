use crate::InnerClient;
use std::sync::Arc;

pub trait AsClientSealed {
  fn as_client(&self) -> Arc<InnerClient>;
}

/// Any datatype that can be interpreted as a [`Client`][crate::Client].
pub trait AsClient: AsClientSealed {}

impl AsClientSealed for str {
  #[inline(always)]
  fn as_client(&self) -> Arc<InnerClient> {
    Arc::new(InnerClient::new(String::from(self)))
  }
}

impl AsClient for str {}
