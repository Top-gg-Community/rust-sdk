# topgg [![crates.io][crates-io-image]][crates-io-url] [![crates.io downloads][crates-io-downloads-image]][crates-io-url] [![license][github-license-image]][github-license-url] [![BLAZINGLY FAST!!!][blazingly-fast-image]][blazingly-fast-url]

[crates-io-image]: https://img.shields.io/crates/v/topgg?style=flat-square
[crates-io-downloads-image]: https://img.shields.io/crates/d/topgg?style=flat-square
[crates-io-url]: https://crates.io/crates/topgg
[github-license-image]: https://img.shields.io/github/license/top-gg/rust-sdk?style=flat-square
[github-license-url]: https://github.com/top-gg/rust-sdk/blob/main/LICENSE
[blazingly-fast-image]: https://img.shields.io/badge/speed-BLAZINGLY%20FAST!!!%20%F0%9F%94%A5%F0%9F%9A%80%F0%9F%92%AA%F0%9F%98%8E-brightgreen.svg?style=flat-square
[blazingly-fast-url]: https://twitter.com/acdlite/status/974390255393505280
The official Rust SDK for the [Top.gg API](https://docs.top.gg).

## Getting Started

Make sure to have a [Top.gg](https://top.gg) API token handy, you can have an API token if you own a listed Discord bot on [Top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:

```toml
topgg = "1.0"
```

More things can be read on the [documentation](https://docs.rs/topgg).

## Features

This library provides several feature flags that can be enabled/disabled in `Cargo.toml`. Such as:

- **`api`**: Interacting with the [Top.gg](https://top.gg) API and accessing the `top.gg/api/*` endpoints. (enabled by default)
  - **`autoposter`**: Automating the process of periodically posting bot statistics to the [Top.gg](https://top.gg) API.
- **`webhook`**: Accessing the [`serde` deserializable](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html) `topgg::Vote` struct.
  - **`actix`**: Wrapper for working with the [`actix-web`](https://crates.io/crates/actix-web) web framework.
  - **`axum`**: Wrapper for working with the [`axum`](https://crates.io/crates/axum) web framework.
  - **`rocket`**: Wrapper for working with the [`rocket`](https://rocket.rs/) web framework.
  - **`warp`**: Wrapper for working with the [`warp`](https://crates.io/crates/warp) web framework.

## Examples

<details>
<summary><b><code>api</code></b>: Fetching a single Discord user from it's Discord ID</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
  let client = Client::new(token);
  
  let user = client.get_user(661200758510977084u64).await.unwrap();
  
  assert_eq!(user.username, "null");
  assert_eq!(user.discriminator, "8626");
  assert_eq!(user.id, 661200758510977084u64);
  
  println!("{:?}", user);
}
```

</details>
<details>
<summary><b><code>api</code></b>: Fetching a single Discord bot from it's Discord ID</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
  let client = Client::new(token);
  
  let bot = client.get_bot(264811613708746752u64).await.unwrap();
  
  assert_eq!(bot.username, "Luca");
  assert_eq!(bot.discriminator, "1375");
  assert_eq!(bot.id, 264811613708746752u64);
  
  println!("{:?}", bot);
}
```

</details>
<details>
<summary><b><code>api</code></b>: Querying several Discord bots</summary>

```rust,no_run
use topgg::{Client, Filter, Query};

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
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

</details>
<details>
<summary><b><code>api</code></b>: Posting your Discord bot's statistics</summary>

```rust,no_run
use topgg::{Client, NewBotStats};

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
  let client = Client::new(token);

  let server_count = 1234; // be TRUTHFUL!
  let shard_count = 10;

  let stats = NewBotStats::count_based(server_count, Some(shard_count));

  client.post_stats(stats).await.unwrap();
}
```

</details>
<details>
<summary><b><code>api</code></b>: Checking if a user has voted for your Discord bot</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
  let client = Client::new(token);

  if client.has_voted(661200758510977084u64).await.unwrap() {
    println!("checks out");
  }
}
```

</details>
<details>
<summary><b><code>autoposter</code></b>: Automating the process of periodically posting your Discord bot's statistics</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.0", features = ["autoposter"] }
```

In your code:

```rust,no_run
use topgg::{Autoposter, Client, NewBotStats};

#[tokio::main]
async fn main() {
  let token = env!("TOPGG_TOKEN").to_owned();
  let client = Client::new(token);

  // make sure to make this autoposter instance live
  // throughout most of the bot's lifetime to keep running!
  let autoposter = client.new_autoposter(1800);

  // ... then in some on ready/new guild event ...
  let server_count = 12345;
  let stats = NewBotStats::count_based(server_count, None);
  autoposter.feed(stats).await;
}
```

</details>
<details>
<summary><b><code>actix</code></b>: Writing an <a href="https://crates.io/crates/actix-web"><code>actix-web</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.0", default-features = false, features = ["actix"] }
```

In your code:

```rust,no_run
use actix_web::{post, App, HttpServer, Responder};
use std::io;

#[post("/dblwebhook")]
async fn webhook(vote: topgg::IncomingVote) -> impl Responder {
  match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
    Some(vote) => /* your application logic here... */,
    _ => /* handle 401 here... */,
  }
}

#[tokio::main]
async fn main() -> io::Result<()> {
  HttpServer::new(|| {
    App::new().service(webhook)
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}
```

</details>
<details>
<summary><b><code>axum</code></b>: Writing an <a href="https://crates.io/crates/axum"><code>axum</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.0", default-features = false, features = ["axum"] }
```

In your code:

```rust,no_run
use axum::{Router, Server};
use std::net::SocketAddr;

struct MyVoteHandler {}

#[async_trait::async_trait]
impl topgg::VoteHandler for MyVoteHandler {
  async fn voted(&self, vote: topgg::Vote) {
    // your application logic here
  }
}

#[tokio::main]
async fn main() {
  let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
  let state = MyVoteHandler {};
  
  let app = Router::new()
    .nest("/dblwebhook", topgg::axum::webhook(password, state));
  
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}
```

</details>
<details>
<summary><b><code>rocket</code></b>: Writing an <a href="https://rocket.rs"><code>rocket</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.0", default-features = false, features = ["rocket"] }
```

In your code:

```rust,no_run
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::Status;

#[post("/", data = "<vote>")]
fn webhook(vote: topgg::IncomingVote) -> Status {
  match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
    Some(vote) => /* your application logic here... */,
    _ => /* handle 401 here... */,
  }
}

fn main() {
  rocket::ignite()
    .mount("/dblwebhook", routes![webhook])
    .launch();
}
```

</details>
<details>
<summary><b><code>warp</code></b>: Writing an <a href="https://crates.io/crates/warp"><code>warp</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.0", default-features = false, features = ["warp"] }
```

In your code:

```rust,no_run
struct MyVoteHandler {}

#[async_trait::async_trait]
impl topgg::VoteHandler for MyVoteHandler {
  async fn voted(&self, vote: topgg::Vote) {
    // your application logic here
  }
}

#[tokio::main]
async fn main() {
  let password = env!("TOPGG_WEBHOOK_PASSWORD").to_owned();
  let state = MyVoteHandler {};
  
  // POST /dblwebhook
  let webhook = topgg::warp::webhook("dblwebhook", password, state);   
  let routes = warp::post().and(webhook);

  warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

</details>