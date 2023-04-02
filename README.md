# topgg

The official Rust SDK for the [Top.gg API](https://docs.top.gg).

## Getting Started

Make sure to have a top.gg API token handy, you can have a token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:

```toml
topgg = "0.1"
```

More things can be read on [the documentation](https://docs.rs/topgg).

## Examples

- Fetching a single discord user from it's Discord ID

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  
  let best_user_of_all_time = client.get_user(661200758510977084u64).await.unwrap();
  
  assert_eq!(best_user_of_all_time.username, "null");
  assert_eq!(best_user_of_all_time.discriminator, "8626");
  assert_eq!(best_user_of_all_time.id, 661200758510977084u64);
  
  println!("{:?}", best_user_of_all_time);
}
```

- Fetching a single discord bot from it's Discord ID

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  
  let bot = client.get_bot(282859044593598464u64).await.unwrap();
  
  assert_eq!(bot.username, "ProBot ✨");
  assert_eq!(bot.id, 282859044593598464u64);
  
  println!("{:?}", bot);
}
```

- Querying a discord bot from their username

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  
  for bot in client.get_bots("shiro").await.unwrap() {
    println!("{:?}", bot);
  }
}
```

- Querying a discord bot with advanced configurations

```rust,no_run
use topgg::{Client, Filter, Query};

#[tokio::main]
async fn main() {
  let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  
  let filter = Filter::new()
    .username("shiro")
	.certified(true);
  
  let query = Query::new()
    .limit(100)
    .filter(filter);
  
  for bot in client.get_bots(query).await.unwrap() {
    println!("{:?}", bot);
  }
}
```