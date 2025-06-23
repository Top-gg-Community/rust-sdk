# Top.gg Rust SDK

The community-maintained Rust library for Top.gg.

## Installation

In your `Cargo.toml`:

```toml
[dependencies]
topgg = "2"
```

## Setting up

```rust,no_run
use topgg::Client;

let client = Client::new(env!("TOPGG_TOKEN").to_string());
```

## Usage

### Getting a bot

```rust,no_run
let bot = client.get_bot(264811613708746752).await.unwrap();
```

### Getting several bots

```rust,no_run
let bots = client
  .get_bots()
  .limit(250)
  .skip(50)
  .sort_by_monthly_votes()
  .await
  .unwrap();

for bot in bots {
  println!("{}", bot.name);
}
```

### Getting your bot's voters

```rust,no_run
//                             Page number
let voters = client.get_voters(1).await.unwrap();

for voter in voters {
  println!("{}", voter.username);
}
```

### Check if a user has voted for your bot

```rust,no_run
let has_voted = client.has_voted(661200758510977084).await.unwrap();
```

### Getting your bot's server count

```rust,no_run
let server_count = client.get_server_count().await.unwrap();
```

### Posting your bot's server count

```rust,no_run
client.post_server_count(bot.server_count()).await.unwrap();
```

### Automatically posting your bot's server count every few minutes

#### Serenity

In your `Cargo.toml`:

```toml
[dependencies]
# using serenity with guild caching disabled
topgg = { version = "2", features = ["autoposter", "serenity"] }

# using serenity with guild caching enabled
topgg = { version = "2", features = ["autoposter", "serenity-cached"] }
```

In your code:

```rust,no_run
use std::time::Duration;
use serenity::{client::{Client, Context, EventHandler}, model::gateway::{GatewayIntents, Ready}};
use topgg::Autoposter;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is now ready!", ready.user.name);
  }
}

#[tokio::main]
async fn main() {
  let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());

  // Posts once every 30 minutes
  let mut autoposter = Autoposter::serenity(&client, Duration::from_secs(1800));
  
  let bot_token = env!("BOT_TOKEN").to_string();
  let intents = GatewayIntents::GUILDS;

  let mut bot = Client::builder(&bot_token, intents)
    .event_handler(Handler)
    .event_handler_arc(autoposter.handler())
    .await
    .unwrap();

  let mut receiver = autoposter.receiver();

  tokio::spawn(async move {
    while let Some(result) = receiver.recv().await {
      println!("Just posted: {result:?}");
    }
  });
  
  if let Err(why) = bot.start().await {
    println!("Client error: {why:?}");
  }
}
```

#### Twilight

In your `Cargo.toml`:

```toml
[dependencies]
# using twilight with guild caching disabled
topgg = { version = "2", features = ["autoposter", "twilight"] }

# using twilight with guild caching enabled
topgg = { version = "2", features = ["autoposter", "twilight-cached"] }
```

In your code:

```rust,no_run
use std::time::Duration;
use topgg::{Autoposter, Client};
use twilight_gateway::{Event, Intents, Shard, ShardId};

#[tokio::main]
async fn main() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());
  let autoposter = Autoposter::twilight(&client, Duration::from_secs(1800));

  let mut shard = Shard::new(
    ShardId::ONE,
    env!("DISCORD_TOKEN").to_string(),
    Intents::GUILD_MEMBERS | Intents::GUILDS,
  );

  loop {
    let event = match shard.next_event().await {
      Ok(event) => event,
      Err(source) => {
        if source.is_fatal() {
          break;
        }

        continue;
      }
    };
    
    autoposter.handle(&event).await;
    
    match event {
      Event::Ready(_) => {
        println!("Bot is now ready!");
      },

      _ => {}
    }
  }
}
```

### Checking if the weekend vote multiplier is active

```rust,no_run
let is_weekend = client.is_weekend().await.unwrap();
```

### Generating widget URLs

#### Large

```rust,no_run
use topgg::{Widget, WidgetType};

let widget_url = Widget::large(WidgetType::DiscordBot, 574652751745777665);
```

#### Votes

```rust,no_run
use topgg::{Widget, WidgetType};

let widget_url = Widget::votes(WidgetType::DiscordBot, 574652751745777665);
```

#### Owner

```rust,no_run
use topgg::{Widget, WidgetType};

let widget_url = Widget::owner(WidgetType::DiscordBot, 574652751745777665);
```

#### Social

```rust,no_run
use topgg::{Widget, WidgetType};

let widget_url = Widget::social(WidgetType::DiscordBot, 574652751745777665);
```

### Webhooks

#### Being notified whenever someone voted for your bot

##### actix-web

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["actix-web"] }
```

In your code:

```rust,no_run
use actix_web::{
  error::{Error, ErrorUnauthorized},
  get, post, App, HttpServer,
};
use topgg::{Incoming, Vote};
use std::io;

#[post("/votes")]
async fn voted(vote: Incoming<Vote>) -> Result<&'static str, Error> {
  match vote.authenticate(env!("MY_TOPGG_WEBHOOK_SECRET")) {
    Some(vote) => {
      println!("A user with the ID of {} has voted us on Top.gg!", vote.voter_id);

      Ok("ok")
    },
    _ => Err(ErrorUnauthorized("401")),
  }
}

#[get("/")]
async fn index() -> &'static str {
  "Hello, World!"
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  HttpServer::new(|| App::new().service(index).service(voted))
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

##### axum

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["axum"] }
```

In your code:

```rust,no_run
use axum::{routing::get, Router};
use topgg::{Vote, Webhook};
use tokio::net::TcpListener;
use std::sync::Arc;

struct MyVoteListener {}

#[async_trait::async_trait]
impl Webhook<Vote> for MyVoteListener {
  async fn callback(&self, vote: Vote) {
    println!("A user with the ID of {} has voted us on Top.gg!", vote.voter_id);
  }
}

async fn index() -> &'static str {
  "Hello, World!"
}

#[tokio::main]
async fn main() {
  let state = Arc::new(MyVoteListener {});

  let router = Router::new().route("/", get(index)).nest(
    "/votes",
    topgg::axum::webhook(env!("MY_TOPGG_WEBHOOK_SECRET").to_string(), Arc::clone(&state)),
  );

  let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

  axum::serve(listener, router).await.unwrap();
}
```

##### rocket

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["rocket"] }
```

In your code:

```rust,no_run
use rocket::{get, http::Status, launch, post, routes};
use topgg::{Incoming, Vote};

#[post("/votes", data = "<vote>")]
fn voted(vote: Incoming<Vote>) -> Status {
  match vote.authenticate(env!("MY_TOPGG_WEBHOOK_SECRET")) {
    Some(vote) => {
      println!("A user with the ID of {} has voted us on Top.gg!", vote.voter_id);

      Status::NoContent
    },
    _ => Status::Unauthorized,
  }
}

#[get("/")]
fn index() -> &'static str {
  "Hello, World!"
}

#[launch]
fn start() -> _ {
  rocket::build().mount("/", routes![index, voted])
}
```

##### warp

In your `Cargo.toml`:

```toml
[dependencies]
topgg = { version = "2", default-features = false, features = ["warp"] }
```

In your code:

```rust,no_run
use std::{net::SocketAddr, sync::Arc};
use topgg::{Vote, Webhook};
use warp::Filter;

struct MyVoteListener {}

#[async_trait::async_trait]
impl Webhook<Vote> for MyVoteListener {
  async fn callback(&self, vote: Vote) {
    println!("A user with the ID of {} has voted us on Top.gg!", vote.voter_id);
  }
}

#[tokio::main]
async fn main() {
  let state = Arc::new(MyVoteListener {});

  // POST /votes
  let webhook = topgg::warp::webhook(
    "votes",
    env!("MY_TOPGG_WEBHOOK_SECRET").to_string(),
    Arc::clone(&state),
  );

  let routes = warp::get().map(|| "Hello, World!").or(webhook);

  let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

  warp::serve(routes).run(addr).await
}
```