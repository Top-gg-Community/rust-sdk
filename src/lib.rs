//! # topgg
//!
//! The official Rust SDK for the [Top.gg API](https://docs.top.gg).
//!
//! ## Getting Started
//!
//! Make sure to have a [top.gg](https://top.gg) API token handy, you can have an API token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:
//!
//! ```toml
//! topgg = "0.1"
//! ```
//!
//! More things can be read on the [documentation](https://docs.rs/topgg).
//!
//! ## Examples
//!
//! - Fetching a single discord user from it's Discord ID
//!
//! ```rust,no_run
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
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
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
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
//! use topgg::{Client, Filter, Query};
//!
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
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
//! - Posting a listed discord bot's statistics
//!
//! ```rust,no_run
//! use topgg::{Client, NewBotStats};
//!
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
//!   let my_bot_id = 123456789u64;
//!
//!   let stats = NewBotStats::new()
//!     .server_count(1234); // be TRUTHFUL!
//!
//!   client.set_bot_stats(my_bot_id, stats).await.unwrap();
//! }
//! ```
//!
//! - Checking if a user has voted for a listed discord bot
//!
//! ```rust,no_run
//! use topgg::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
//!   
//!   let bot_id = 264811613708746752u64;
//!   let user_id = 661200758510977084u64;
//!
//!   if client.has_user_voted(bot_id, user_id).await.unwrap() {
//!     println!("checks out");
//!   }
//! }
//! ```

#![allow(clippy::transmute_int_to_bool)]

mod client;
mod error;

pub(crate) mod http;
pub(crate) mod util;

/// Bot-related structs.
pub mod bot;

/// Snowflake utilities.
pub mod snowflake;

/// User-related structs.
pub mod user;

pub use bot::{Filter, NewBotStats, Query};
pub use client::Client;
pub use error::{Error, InternalError, Result};
