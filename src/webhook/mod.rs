mod vote;
#[cfg_attr(docsrs, doc(cfg(feature = "webhook")))]
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
