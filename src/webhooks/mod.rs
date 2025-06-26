mod vote;
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub use vote::*;

#[cfg(feature = "actix-web")]
mod actix_web;

#[cfg(feature = "rocket")]
mod rocket;

cfg_if::cfg_if! {
  if #[cfg(feature = "axum")] {
    /// Extra helpers for working with axum.
    #[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
    pub mod axum;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "warp")] {
    /// Extra helpers for working with warp.
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    pub mod warp;
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix-web", feature = "rocket"))] {
    /// An unauthenticated incoming Top.gg webhook request.
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "actix-web", feature = "rocket"))))]
    pub struct Incoming<T> {
      pub(crate) authorization: String,
      pub(crate) data: T,
    }

    impl<T> Incoming<T> {
      /// Authenticates a valid password with this request.
      #[must_use]
      #[inline(always)]
      pub fn authenticate(self, password: &str) -> Option<T> {
        if self.authorization == password {
          Some(self.data)
        } else {
          None
        }
      }
    }

    impl<T> Clone for Incoming<T>
    where
      T: Clone,
    {
      #[inline(always)]
      fn clone(&self) -> Self {
        Self {
          authorization: self.authorization.clone(),
          data: self.data.clone(),
        }
      }
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "axum", feature = "warp"))] {
    /// Webhook event handler.
    #[cfg_attr(docsrs, doc(cfg(any(feature = "axum", feature = "warp"))))]
    #[async_trait::async_trait]
    pub trait Webhook<T>: Send + Sync + 'static {
      async fn callback(&self, data: T);
    }
  }
}
