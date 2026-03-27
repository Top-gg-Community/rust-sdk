#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::needless_pass_by_value)]

mod project;
mod snowflake;
#[cfg(test)]
mod test;
mod user;

pub use project::*;
pub use user::*;

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    mod client;
    mod error;
    mod util;

    pub use client::Client;
    pub use error::{Error, PostCommandsError, PostCommandsResult, Result};
    pub use snowflake::Snowflake; // for doc purposes

    /// Widget generator functions.
    pub mod widget;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "webhooks")] {
    mod webhooks;

    pub use webhooks::*;
  }
}
