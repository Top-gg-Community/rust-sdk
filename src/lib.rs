//! # topgg [![crates.io][crates-io-image]][crates-io-url] [![crates.io downloads][crates-io-downloads-image]][crates-io-url] [![license][github-license-image]][github-license-url] [![BLAZINGLY FAST!!!][blazingly-fast-image]][blazingly-fast-url]
//!
//! [crates-io-image]: https://img.shields.io/crates/v/topgg?style=flat-square
//! [crates-io-downloads-image]: https://img.shields.io/crates/d/topgg?style=flat-square
//! [crates-io-url]: https://crates.io/crates/topgg
//! [github-license-image]: https://img.shields.io/github/license/top-gg/rust-sdk?style=flat-square
//! [github-license-url]: https://github.com/top-gg/rust-sdk/blob/main/LICENSE
//! [blazingly-fast-image]: https://img.shields.io/badge/speed-BLAZINGLY%20FAST!!!%20%F0%9F%94%A5%F0%9F%9A%80%F0%9F%92%AA%F0%9F%98%8E-brightgreen.svg?style=flat-square
//! [blazingly-fast-url]: https://twitter.com/acdlite/status/974390255393505280
//! The official Rust SDK for the [Top.gg API](https://docs.top.gg).
//!
//! ## Getting Started
//!
//! Make sure to have a [top.gg](https://top.gg) API token handy, you can have an API token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:
//!
//! ```toml
//! topgg = "1.0"
//! ```
//!
//! More things can be read on the [documentation](https://docs.rs/topgg).
//!
//! ## Examples
//!
//! - Fetching a single discord user from it's Discord ID
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   
//!   let user = client.get_user(661200758510977084u64).await.unwrap();
//!   
//!   assert_eq!(user.username, "null");
//!   assert_eq!(user.discriminator, "8626");
//!   assert_eq!(user.id, 661200758510977084u64);
//!   
//!   println!("{:?}", user);
//! }
//! ```
//!
//! - Fetching a single discord bot from it's Discord ID
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   
//!   let bot = client.get_bot(264811613708746752u64).await.unwrap();
//!   
//!   assert_eq!(bot.username, "Luca");
//!   assert_eq!(bot.discriminator, "1375");
//!   assert_eq!(bot.id, 264811613708746752u64);
//!   
//!   println!("{:?}", bot);
//! }
//! ```
//!
//! - Querying several discord bots
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::{Client, Filter, Query};
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   
//!   // inputting a string searches a bot that matches that username
//!   for bot in client.get_bots("shiro").await.unwrap() {
//!     println!("{:?}", bot);
//!   }
//!
//!   // advanced query with filters
//!   let filter = Filter::new()
//!     .username("shiro")
//!     .certified(true);
//!
//!   let query = Query::new()
//!     .limit(250)
//!     .skip(50)
//!     .filter(filter);
//!
//!   for bot in client.get_bots(query).await.unwrap() {
//!     println!("{:?}", bot);
//!   }
//! }
//! ```
//!
//! - Posting an owned discord bot's statistics
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::{Client, NewBotStats};
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   let my_bot_id = 123456789u64;
//!
//!   let server_count = 1234; // be TRUTHFUL!
//!   let shard_count = 10;
//!
//!   let stats = NewBotStats::count_based(server_count, Some(shard_count));
//!
//!   client.post_bot_stats(my_bot_id, stats).await.unwrap();
//! }
//! ```
//!
//! - Posting a listed discord bot's statistics (with an autoposter)
//!
//! > **NOTE:** this requires the `autoposter` feature to be enabled.
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::{Autoposter, Client, NewBotStats};
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   let my_bot_id = 123456789u64;
//!
//!   // make sure to make this autoposter instance live
//!   // throughout most of the bot's lifetime to keep running!
//!   let autoposter = client.new_autoposter(my_bot_id, 1800);
//!
//!   // ... then in some on ready/new guild event ...
//!   let server_count = 12345;
//!   let stats = NewBotStats::count_based(server_count, None);
//!   autoposter.feed(stats).await;
//! }
//! ```
//!
//! - Checking if a user has voted for a listed discord bot
//!
//! ```rust,no_run
//! use std::env;
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
//!   let client = Client::new(token);
//!   
//!   let my_bot_id = 123456789u64;
//!   let user_id = 661200758510977084u64;
//!
//!   if client.has_voted(my_bot_id, user_id).await.unwrap() {
//!     println!("checks out");
//!   }
//! }
//! ```

#![allow(clippy::transmute_int_to_bool)]
#![doc(
  html_logo_url = "https://top.gg/favicon.png",
  html_favicon_url = "https://top.gg/favicon.png"
)]

mod client;
mod error;
mod http;
mod util;

/// Bot-related structs.
pub mod bot;

/// Snowflake utilities.
pub mod snowflake;

/// User-related structs.
pub mod user;

pub use bot::{Filter, NewBotStats, Query};
pub use client::Client;
pub use error::{Error, InternalError, Result};

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    mod autoposter;
    pub use autoposter::Autoposter;
  }
}
