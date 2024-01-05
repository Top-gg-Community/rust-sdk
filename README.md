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

Make sure to have a [Top.gg API](https://docs.top.gg) token handy. If not, then [view this tutorial on how to retrieve yours](https://github.com/top-gg/rust-sdk/assets/60427892/d2df5bd3-bc48-464c-b878-a04121727bff). After that, add the following line to the `dependencies` section of your `Cargo.toml`:

```toml
topgg = "1.3"
```

## Features

This library provides several feature flags that can be enabled/disabled in `Cargo.toml`. Such as:

- **`api`**: Interacting with the [Top.gg API](https://docs.top.gg) and accessing the `top.gg/api/*` endpoints. (enabled by default)
  - **`autoposter`**: Automating the process of periodically posting bot statistics to the [Top.gg API](https://docs.top.gg).
- **`webhook`**: Accessing the [`serde` deserializable](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html) `topgg::Vote` struct.
  - **`actix-web`**: Wrapper for working with the [`actix-web`](https://actix.rs/) web framework.
  - **`axum`**: Wrapper for working with the [`axum`](https://crates.io/crates/axum) web framework.
  - **`rocket`**: Wrapper for working with the [`rocket`](https://rocket.rs/) web framework.
  - **`warp`**: Wrapper for working with the [`warp`](https://crates.io/crates/warp) web framework.
- **`serenity`**: Extra helpers for working with [`serenity`](https://crates.io/crates/serenity) library (with bot caching disabled).
  - **`serenity-cached`**: Extra helpers for working with [`serenity`](https://crates.io/crates/serenity) library (with bot caching enabled).
- **`twilight`**: Extra helpers for working with [`twilight`](https://crates.io/crates/twilight) library (with bot caching disabled).
  - **`twilight-cached`**: Extra helpers for working with [`twilight`](https://crates.io/crates/serenity) library (with bot caching enabled).

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
use topgg::{Client, Query};

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  // inputting a string searches a bot that matches that username.
  for bot in client.get_bots("shiro").await.unwrap() {
    println!("{:?}", bot);
  }

  let query = Query::new()
    .limit(250)
    .skip(50)
    .username("shiro")
    .certified(true);

  for bot in client.get_bots(query).await.unwrap() {
    println!("{:?}", bot);
  }
}
```

</details>
<details>
<summary><b><code>api</code></b>: Posting your Discord bot's statistics</summary>

```rust,no_run
use topgg::{Client, Stats};

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  let server_count = 12345;
  client
    .post_stats(Stats::from(server_count))
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
<summary><b><code>autoposter</code></b>, <b><code>serenity</code></b>: Automating the process of periodically posting your Discord bot's statistics with the <i>serenity</i> library</summary>

In your `Cargo.toml`:

```toml
[dependencies]
# using serenity with guild caching disabled
topgg = { version = "1.3", features = ["autoposter", "serenity"] }

# using serenity with guild caching enabled
topgg = { version = "1.3", features = ["autoposter", "serenity-cached"] }
```

In your code:

```rust,no_run
use core::time::Duration;
use serenity::{client::{Client, Context, EventHandler}, model::{channel::Message, gateway::Ready}};
use topgg::{Autoposter, Stats};

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.content == "!ping" {
      if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        println!("Error sending message: {why:?}");
      }
    }
  }

  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
  }
}

#[tokio::main]
async fn main() {
  // if you also want to communicate with the Top.gg API, you can do this
  let topgg_client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  let autoposter = Autoposter::serenity(&topgg_client, Duration::from_secs(1800));
  
  // otherwise you can do this
  let topgg_token = env!("TOPGG_TOKEN").to_string();
  let autoposter = Autoposter::serenity(&topgg_token, Duration::from_secs(1800));
  
  let bot_token = env!("DISCORD_TOKEN").to_string();
  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS | GatewayIntents::MESSAGE_CONTENT;

  let mut client = Client::builder(&bot_token, intents)
    .event_handler(Handler)
    .event_handler_arc(autoposter.handler())
    .await
    .unwrap();

  if let Err(why) = client.start().await {
    println!("Client error: {why:?}");
  }
}
```

</details>
<details>
<summary><b><code>actix-web</code></b>: Writing an <a href="https://actix.rs/"><code>actix-web</code></a> webhook for listening to your bot/server's vote events</summary>

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "1.3", default-features = false, features = ["actix-web"] }
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
topgg = { version = "1.3", default-features = false, features = ["axum"] }
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
    topgg::axum::webhook(env!("TOPGG_WEBHOOK_PASSWORD").to_string(), Arc::clone(&state)),
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
topgg = { version = "1.3", default-features = false, features = ["rocket"] }
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
topgg = { version = "1.3", default-features = false, features = ["warp"] }
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
    Arc::clone(&state),
  );

  let routes = warp::get().map(|| "Hello, World!").or(webhook);

  let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

  warp::serve(routes).run(addr).await
}
```

</details>
