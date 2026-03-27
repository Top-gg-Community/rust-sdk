# [Top.gg Rust SDK](https://crates.io/crates/topgg) [![crates.io][crates-io-image]][crates-io-url] [![crates.io downloads][crates-io-downloads-image]][crates-io-url]

[crates-io-image]: https://img.shields.io/crates/v/topgg?style=flat-square
[crates-io-downloads-image]: https://img.shields.io/crates/d/topgg?style=flat-square
[crates-io-url]: https://crates.io/crates/topgg

> For more information, see the documentation here: <https://docs.rs/topgg>.

The community-maintained Rust SDK for Top.gg.

## Chapters

- [Installation](#installation)
- [Features](#features)
- [Setting up](#setting-up)
- [Usage](#usage)
  - [Getting your project's information](#getting-your-projects-information)
  - [Getting your project's vote information of a user](#getting-your-projects-vote-information-of-a-user)
  - [Getting a cursor-based paginated list of votes for your project](#getting-a-cursor-based-paginated-list-of-votes-for-your-project)
  - [Posting your bot's application commands list](#posting-your-bots-application-commands-list)
  - [Generating widget URLs](#generating-widget-urls)
  - [Webhooks](#webhooks)

## Installation

Add the following line to the `dependencies` section of your `Cargo.toml`:

```toml
topgg = "2"
```

## Features

This SDK provides several feature flags that can be enabled/disabled in `Cargo.toml`, such as:

- **`api`**: Interacting with the [Top.gg API](https://docs.top.gg) and accessing the `top.gg/api/*` endpoints. (enabled by default)
- **`webhooks`**: Accessing [serde deserializable](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html) webhook payload structs.
  - **`actix-web`**: Wrapper for working with the [actix-web](https://actix.rs/) web framework.
  - **`axum`**: Wrapper for working with the [axum](https://crates.io/crates/axum) web framework.
  - **`rocket`**: Wrapper for working with the [rocket](https://rocket.rs/) web framework.
  - **`warp`**: Wrapper for working with the [warp](https://crates.io/crates/warp) web framework.
- **`serenity`**: Extra helpers for working with [serenity](https://crates.io/crates/serenity).
- **`twilight`**: Extra helpers for working with [twilight](https://twilight.rs).

## Setting up

```rust,no_run
let client = topgg::Client::new(env!("TOPGG_TOKEN").into());
```

## Usage

### Getting your project's information

```rust,no_run
let project = client.get_self().await.unwrap();
```

### Getting your project's vote information of a user

#### Discord ID

```rust,no_run
let vote = client.get_vote(UserSource::Discord(661200758510977084)).await.unwrap();
```

#### Top.gg ID

```rust,no_run
let vote = client.get_vote(UserSource::Topgg(8226924471638491136)).await.unwrap();
```

### Getting a cursor-based paginated list of votes for your project

```rust,no_run
use chrono::{TimeZone, Utc};

let since = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();
let first_page = client.get_votes(since).await.unwrap();

for vote in first_page.iter() {
  println!("{vote:?}");
}

let second_page = first_page.next().await.unwrap();

for vote in second_page.iter() {
  println!("{vote:?}");
}
```

### Posting your bot's application commands list

#### Serenity

```rust,no_run
client.post_commands(&ctx).await.unwrap();
```

#### Twilight

```rust,no_run
let application_id = bot.current_user_application().await.unwrap().model().await.unwrap().id;
let interaction = bot.interaction(application_id);

client.post_commands(interaction.global_commands()).await.unwrap();
```

#### Raw

```rust,no_run
let commands = json!([{
  "id": "1",
  "type": 1,
  "application_id": "1",
  "name": "test",
  "description": "command description",
  "default_member_permissions": "",
  "version": "1"
}]); // Array of application commands that
     // can be serialized to Discord API's raw JSON format.

client.post_commands(commands).await.unwrap();
```

### Generating widget URLs

#### Large

```rust,no_run
let widget_url = topgg::widget::large(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
```

#### Votes

```rust,no_run
let widget_url = topgg::widget::votes(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
```

#### Owner

```rust,no_run
let widget_url = topgg::widget::owner(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
```

#### Social

```rust,no_run
let widget_url = topgg::widget::social(topgg::Platform::Discord, topgg::ProjectType::Bot, 1026525568344264724);
```

### Webhooks

#### Actix-web

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["actix-web"] }
```

In your code:

```rust,no_run
use topgg::{IncomingPayload, PayloadResult};
use std::io;

use actix_web::{
  App, HttpServer,
  error::{Error, ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
  get, post,
};

#[get("/")]
async fn index() -> &'static str {
  "Hello, World!"
}

// POST /webhook
#[post("/webhook")]
async fn webhook(payload: IncomingPayload) -> Result<&'static str, Error> {
  match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
    PayloadResult::Accepted(payload) => {
      println!("{payload:?}");

      Ok("ok")
    }

    PayloadResult::Forbidden => Err(ErrorForbidden("Forbidden")),

    PayloadResult::BadRequest => Err(ErrorBadRequest("Bad Request")),

    PayloadResult::Unauthorized => Err(ErrorUnauthorized("Unauthorized")),

    PayloadResult::DeserializationFailure => Ok(""),

    PayloadResult::InternalServerError => Err(ErrorInternalServerError("Internal Server Error")),
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

#### Axum

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["axum"] }
```

In your code:

```rust,no_run
use topgg::Payload;
use std::sync::Arc;

use axum::{
  Router,
  http::status::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
};
use tokio::net::TcpListener;

struct MyTopggListener {}

#[async_trait::async_trait]
impl topgg::axum::Listener for MyTopggListener {
  async fn callback(self: Arc<Self>, payload: Payload, _trace: &str) -> Response {
    println!("{payload:?}");

    (StatusCode::NO_CONTENT, ()).into_response()
  }
}

async fn index() -> &'static str {
  "Hello, World!"
}

#[tokio::main]
async fn main() {
  let state = Arc::new(MyTopggListener {});

  // POST /webhook
  let router = Router::new().route("/", get(index)).nest(
    "/webhook",
    topgg::axum::webhook(Arc::clone(&state), env!("TOPGG_WEBHOOK_SECRET").into()),
  );

  let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

  axum::serve(listener, router).await.unwrap();
}
```

#### Rocket

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["rocket"] }
```

In your code:

```rust,no_run
use topgg::{IncomingPayload, PayloadResult};

use rocket::{Build, Rocket, get, http::Status, launch, post, routes};

#[get("/")]
fn index() -> &'static str {
  "Hello, World!"
}

// POST /webhook
#[post("/webhook", data = "<payload>")]
fn webhook(payload: IncomingPayload) -> Status {
  match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
    PayloadResult::Accepted(payload) => {
      println!("{payload:?}");

      Status::NoContent
    }

    PayloadResult::Forbidden => Status::Forbidden,

    PayloadResult::BadRequest => Status::BadRequest,

    PayloadResult::Unauthorized => Status::Unauthorized,

    PayloadResult::DeserializationFailure => Status::NoContent,

    PayloadResult::InternalServerError => Status::InternalServerError,
  }
}

#[launch]
fn rocket() -> Rocket<Build> {
  rocket::build().mount("/", routes![index, webhook])
}
```

#### Warp

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["warp"] }
```

In your code:

```rust,no_run
use topgg::PayloadResult;
use std::net::SocketAddr;

use warp::{Filter, http::StatusCode, reply};

#[tokio::main]
async fn main() {
  // POST /webhook
  let webhook =
    topgg::warp::webhook("webhook", env!("TOPGG_WEBHOOK_SECRET").into()).then(|payload, _trace| async move {
      match payload {
        PayloadResult::Accepted(payload) => {
          println!("{payload:?}");

          reply::with_status("", StatusCode::NO_CONTENT)
        }

        PayloadResult::Forbidden => reply::with_status("Forbidden", StatusCode::FORBIDDEN),

        PayloadResult::BadRequest => reply::with_status("Bad Request", StatusCode::BAD_REQUEST),

        PayloadResult::Unauthorized => reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED),

        PayloadResult::DeserializationFailure => reply::with_status("", StatusCode::NO_CONTENT),

        PayloadResult::InternalServerError => {
          reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR)
        }
      }
    });

  let routes = warp::get().map(|| "Hello, World!").or(webhook);

  let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

  warp::serve(routes).run(addr).await
}
```
