# topgg

The official Rust SDK for the [Top.gg API](https://docs.top.gg).

## Getting Started

Make sure to have a [top.gg](https://top.gg) API token handy, you can have an API token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:

```toml
topgg = "0.1"
```

More things can be read on the [documentation](https://docs.rs/topgg).

## Examples

- Fetching a single discord user from it's Discord ID

```rust,no_run
use std::env;
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  
  let user = client.get_user(661200758510977084u64).await.unwrap();
  
  assert_eq!(user.username, "null");
  assert_eq!(user.discriminator, "8626");
  assert_eq!(user.id, 661200758510977084u64);
  
  println!("{:?}", user);
}
```

- Fetching a single discord bot from it's Discord ID

```rust,no_run
use std::env;
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  
  let bot = client.get_bot(264811613708746752u64).await.unwrap();
  
  assert_eq!(bot.username, "Luca");
  assert_eq!(bot.discriminator, "1375");
  assert_eq!(bot.id, 264811613708746752u64);
  
  println!("{:?}", bot);
}
```

- Querying several discord bots

```rust,no_run
use std::env;
use topgg::{Client, Filter, Query};

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  
  // inputting a string searches a bot that matches that username
  for bot in client.get_bots("shiro").await.unwrap() {
    println!("{:?}", bot);
  }

  // advanced query with filters
  let filter = Filter::new()
    .username("shiro")
    .certified(true);

  let query = Query::new()
    .limit(250)
    .skip(50)
    .filter(filter);

  for bot in client.get_bots(query).await.unwrap() {
    println!("{:?}", bot);
  }
}
```

- Posting a listed discord bot's statistics

```rust,no_run
use std::env;
use topgg::{Client, NewBotStats};

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  let my_bot_id = 123456789u64;

  let server_count = 1234; // be TRUTHFUL!
  let shard_count = 10;

  let stats = NewBotStats::count_based(server_count, Some(shard_count));

  client.post_bot_stats(my_bot_id, stats).await.unwrap();
}
```

- Posting a listed discord bot's statistics (with an autoposter)

> **NOTE:** this requires the `autoposter` feature to be enabled.

```rust,no_run
use std::env;
use topgg::{Autoposter, Client, NewBotStats};

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  let my_bot_id = 123456789u64;

  // make sure to make this autoposter instance live
  // throughout most of the bot's lifetime to keep running!
  let autoposter = client.new_autoposter(my_bot_id, 1800);

  // ... then in some on ready/new guild event ...
  let server_count = 12345;
  let stats = NewBotStats::count_based(server_count, None);
  autoposter.feed(stats).await;
}
```

- Checking if a user has voted for a listed discord bot

```rust,no_run
use std::env;
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  let client = Client::new(token);
  
  let bot_id = 264811613708746752u64;
  let user_id = 661200758510977084u64;

  if client.has_user_voted(bot_id, user_id).await.unwrap() {
    println!("checks out");
  }
}
```