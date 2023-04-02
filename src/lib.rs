//! # topgg
//! 
//! The official Rust SDK for the [Top.gg API](https://docs.top.gg).
//! 
//! ## Getting Started
//! 
//! Make sure to have a top.gg API token handy, you can have a token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:
//! 
//! ```toml
//! topgg = "0.1"
//! ```
//! 
//! More things can be read on [the documentation](https://docs.rs/topgg).
//! 
//! ## Examples
//! 
//! - Fetching a single bot from it's Discord ID
//! 
//! ```rust,no_run
//! use topgg::Client;
//! 
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
//!   
//!   let bot = client.get_bot(282859044593598464u64).await.unwrap();
//!   
//!   assert_eq!(bot.id, 282859044593598464u64);
//!   assert_eq!(bot.username, "ProBot ✨");
//!   
//!   println!("{:?}", bot);
//! }
//! ```
//! 
//! - Querying a disord bot from their username
//! 
//! ```rust,no_run
//! use topgg::Client;
//! 
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
//!   
//!   for bot in client.get_bots("shiro").await.unwrap() {
//!     println!("{:?}", bot);
//!   }
//! }
//! ```
//! 
//! - Querying a discord with advanced configurations
//! 
//! ```rust,no_run
//! use topgg::{Client, Filter, Query};
//! 
//! #[tokio::main]
//! async fn main() {
//!   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
//!   
//!   let filter = Filter::new()
//!     .username("shiro")
//! 	.certified(true);
//!   
//!   let query = Query::new()
//!     .limit(100)
//!     .filter(filter);
//!   
//!   for bot in client.get_bots(query).await.unwrap() {
//!     println!("{:?}", bot);
//!   }
//! }
//! ```

pub mod bot;
mod client;
mod error;
pub(crate) mod http;
pub mod snowflake;
pub mod user;
pub(crate) mod util;

pub use bot::{Filter, NewBotStats, Query};
pub use client::Client;
pub use error::*;
