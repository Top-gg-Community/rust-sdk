#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "webhooks", allow(unreachable_patterns))]
#![allow(clippy::needless_pass_by_value)]

mod snowflake;
#[cfg(test)]
mod test;

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    pub(crate) mod client;
    mod error;
    mod util;

    #[cfg(feature = "autoposter")]
    pub(crate) use client::InnerClient;

    /// Bot-related traits and structs.
    pub mod bot;

    /// Voter-related structs.
    pub mod voter;

    /// Widget generator functions.
    pub mod widget;

    #[doc(inline)]
    pub use client::Client;
    pub use error::{Error, Result};
    pub use snowflake::Snowflake; // for doc purposes

    #[doc(inline)]
    pub use widget::WidgetType;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    /// Autoposter-related traits and structs.
    #[cfg_attr(docsrs, doc(cfg(feature = "autoposter")))]
    pub mod autoposter;

    #[doc(inline)]
    pub use autoposter::Autoposter;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "webhooks")] {
    mod webhooks;

    pub use webhooks::*;
  }
}
