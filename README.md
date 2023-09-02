# [topgg](https://crates.io/crates/topgg) [![crates.io][crates-io-image]][crates-io-url] [![crates.io downloads][crates-io-downloads-image]][crates-io-url] [![license][github-license-image]][github-license-url] [![BLAZINGLY FAST!!!][blazingly-fast-image]][blazingly-fast-url]

[crates-io-image]: https://img.shields.io/crates/v/topgg?style=flat-square
[crates-io-downloads-image]: https://img.shields.io/crates/d/topgg?style=flat-square
[crates-io-url]: https://crates.io/crates/topgg
[github-license-image]: https://img.shields.io/github/license/top-gg/rust-sdk?style=flat-square
[github-license-url]: https://github.com/top-gg/rust-sdk/blob/main/LICENSE
[blazingly-fast-image]: https://img.shields.io/badge/speed-BLAZINGLY%20FAST!!!%20%F0%9F%94%A5%F0%9F%9A%80%F0%9F%92%AA%F0%9F%98%8E-brightgreen.svg?style=flat-square
[blazingly-fast-url]: https://twitter.com/acdlite/status/974390255393505280
The official Rust SDK for the [Top.gg API](https://docs.top.gg).

## Getting Started

Make sure to have a [Top.gg API](https://docs.top.gg) token handy, you can have an API token if you own a listed Discord bot on [Top.gg](https://top.gg) (open the edit page, see in `Webhooks` section) then add the following to your `Cargo.toml`'s dependencies:

```toml
topgg = "1.2"
```

## Features

This library provides several feature flags that can be enabled/disabled in `Cargo.toml`. Such as:

- **`api`**: Interacting with the [Top.gg](https://top.gg) API and accessing the `top.gg/api/*` endpoints. (enabled by default)
  - **`autoposter`**: Automating the process of periodically posting bot statistics to the [Top.gg](https://top.gg) API.
- **`webhook`**: Accessing the [`serde` deserializable](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html) `topgg::Vote` struct.
  - **`actix-web`**: Wrapper for working with the [`actix-web`](https://actix.rs/) web framework.
  - **`axum`**: Wrapper for working with the [`axum`](https://crates.io/crates/axum) web framework.
  - **`rocket`**: Wrapper for working with the [`rocket`](https://rocket.rs/) web framework.
  - **`warp`**: Wrapper for working with the [`warp`](https://crates.io/crates/warp) web framework.

## Examples

More things can be read in the [documentation](https://docs.rs/topgg).

<details>
<summary><b><code>api</code></b>: Fetching a single Discord user from it's Discord ID</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());
  
  let user = client.get_user(661200758510977084).await.unwrap();
  
  assert_eq!(user.username, "null");
  assert_eq!(user.id, 661200758510977084);
  
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
  let client = Client::new(env!("TOPGG_TOKEN").to_string());
  
  let bot = client.get_bot(264811613708746752).await.unwrap();
  
  assert_eq!(bot.username, "Luca");
  assert_eq!(bot.discriminator, "1375");
  assert_eq!(bot.id, 264811613708746752);
  
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
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  // inputting a string searches a bot that matches that username.
  for bot in client.get_bots("shiro").await.unwrap() {
    println!("{:?}", bot);
  }

  // advanced query with filters...
  let filter = Filter::new().username("shiro").certified(true);

  let query = Query::new().limit(250).skip(50).filter(filter);

  for bot in client.get_bots(query).await.unwrap() {
    println!("{:?}", bot);
  }
}
```

</details>
<details>
<summary><b><code>api</code></b>: Posting your Discord bot's statistics</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  let server_count = 12345;
  client
    .post_stats(NewStats::count_based(server_count, None))
    .await
    .unwrap();
}
```

</details>
<details>
<summary><b><code>api</code></b>: Checking if a user has voted for your Discord bot</summary>

```rust,no_run
use topgg::Client;

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  if client.has_voted(661200758510977084).await.unwrap() {
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
topgg = { version = "1.2", features = ["autoposter"] }
```

In your code:

```rust,no_run
use core::time::Duration;
use topgg::{Autoposter, Client, NewStats};

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  // creates an autoposter that posts data to Top.gg every 1800 seconds (15 minutes).
  // the autopost thread will stop once it's dropped.
  let autoposter = client.new_autoposter(Duration::from_secs(1800));

  // ... then in some on ready/new guild event ...
  let server_count = 12345;
  autoposter
    .feed(NewStats::count_based(server_count, None))
    .await;
}
```

</details>
<details>
<summary><b><code>actix-web</code></b>: Writing an <a href="https://actix.rs/"><code>actix-web</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.2", default-features = false, features = ["actix-web"] }
```

In your code:

```rust,no_run
use actix_web::{
  error::{Error, ErrorUnauthorized},
  get, post, App, HttpServer,
};
use std::io;
use topgg::IncomingVote;

#[get("/")]
async fn index() -> &'static str {
  "Hello, World!"
}

#[post("/webhook")]
async fn webhook(vote: IncomingVote) -> Result<&'static str, Error> {
  match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
    Some(vote) => {
      println!("{:?}", vote);

      Ok("ok")
    }
    _ => Err(ErrorUnauthorized("401")),
  }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  HttpServer::new(|| App::new().service(index).service(webhook))
    .bind("127.0.0.1:8080")?
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
topgg = { version = "1.2", default-features = false, features = ["axum"] }
```

In your code:

```rust,no_run
use axum::{routing::get, Router, Server};
use std::{net::SocketAddr, sync::Arc};
use topgg::{Vote, VoteHandler};

struct MyVoteHandler {}

#[axum::async_trait]
impl VoteHandler for MyVoteHandler {
  async fn voted(&self, vote: Vote) {
    println!("{:?}", vote);
  }
}

async fn index() -> &'static str {
  "Hello, World!"
}

#[tokio::main]
async fn main() {
  let state = Arc::new(MyVoteHandler {});

  let app = Router::new().route("/", get(index)).nest(
    "/webhook",
    topgg::axum::webhook(env!("TOPGG_WEBHOOK_PASSWORD").to_string(), state.clone()),
  );

  let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

  Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}
```

</details>
<details>
<summary><b><code>rocket</code></b>: Writing a <a href="https://rocket.rs"><code>rocket</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.2", default-features = false, features = ["rocket"] }
```

In your code:

```rust,no_run
#![feature(decl_macro)]

use rocket::{get, http::Status, post, routes};
use topgg::IncomingVote;

#[get("/")]
fn index() -> &'static str {
  "Hello, World!"
}

#[post("/webhook", data = "<vote>")]
fn webhook(vote: IncomingVote) -> Status {
  match vote.authenticate(env!("TOPGG_WEBHOOK_PASSWORD")) {
    Some(vote) => {
      println!("{:?}", vote);

      Status::Ok
    },
    _ => {
      println!("found an unauthorized attacker.");

      Status::Unauthorized
    }
  }
}

fn main() {
  rocket::ignite()
    .mount("/", routes![index, webhook])
    .launch();
}
```

</details>
<details>
<summary><b><code>warp</code></b>: Writing a <a href="https://crates.io/crates/warp"><code>warp</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.2", default-features = false, features = ["warp"] }
```

In your code:

```rust,no_run
use std::{net::SocketAddr, sync::Arc};
use topgg::{Vote, VoteHandler};
use warp::Filter;

struct MyVoteHandler {}

#[async_trait::async_trait]
impl VoteHandler for MyVoteHandler {
  async fn voted(&self, vote: Vote) {
    println!("{:?}", vote);
  }
}

#[tokio::main]
async fn main() {
  let state = Arc::new(MyVoteHandler {});

  // POST /webhook
  let webhook = topgg::warp::webhook(
    "webhook",
    env!("TOPGG_WEBHOOK_PASSWORD").to_string(),
    state.clone(),
  );

  let routes = warp::get().map(|| "Hello, World!").or(webhook);

  let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

  warp::serve(routes).run(addr).await
}
```

</details>