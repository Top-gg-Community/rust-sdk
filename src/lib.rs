#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod snowflake;

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    mod client;
    mod error;
    mod util;

    /// Bot-related structs.
    pub mod bot;

    /// User-related structs.
    pub mod user;

    #[doc(inline)]
    pub use bot::{Stats, Query};
    pub use client::Client;
    pub use error::{Error, Result};
    pub use snowflake::Snowflake; // for doc purposes
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    mod autoposter;

    #[cfg_attr(docsrs, doc(cfg(feature = "autoposter")))]
    pub use autoposter::{Autoposter, AutoposterHandle};
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "webhook")] {
    mod webhook;

    pub use webhook::*;
  }
}
