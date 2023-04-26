mod vote;
pub use vote::*;

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "rocket")]
mod rocket;

cfg_if::cfg_if! {
  if #[cfg(feature = "axum")] {
    /// Wrapper for working with the [`axum`](https://crates.io/crates/axum) web framework.
    #[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
    pub mod axum;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "warp")] {
    /// Wrapper for working with the [`warp`](https://crates.io/crates/warp) web framework.
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    pub mod warp;
  }
}
