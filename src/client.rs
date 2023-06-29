use crate::{
  bot::{Bot, Bots, IsWeekend, NewStats, QueryLike, Stats},
  http::{Http, GET, POST},
  user::{User, Voted, Voter},
  Result, SnowflakeLike,
};

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    use core::time::Duration;
    use crate::Autoposter;
    use std::sync::Arc;

    type SyncedClient = Arc<InnerClient>;
  } else {
    type SyncedClient = InnerClient;
  }
}

#[derive(Clone)]
pub(crate) struct InnerClient {
  http: Http,
}

// this is implemented here because autoposter needs to access this function from a different thread

impl InnerClient {
  pub(crate) async fn post_stats(&self, new_stats: &NewStats) -> Result<()> {
    let body = serde_json::to_string(new_stats)?;
    self
      .http
      .send(POST, "/bots/stats", Some(body))
      .await
      .map(|_| ())
  }
}

/// A struct representing a [Top.gg](https://top.gg) API client instance.
#[must_use]
#[derive(Clone)]
pub struct Client {
  inner: SyncedClient,
}

impl Client {
  /// Creates a brand new client instance from a [Top.gg](https://top.gg) token.
  ///
  /// You can get a [Top.gg](https://top.gg) token if you own a listed Discord bot on [Top.gg](https://top.gg) (open the edit page, see in `Webhooks` section)
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let _client = Client::new(env!("TOPGG_TOKEN"));
  /// }
  /// ```
  #[inline(always)]
  pub fn new<T>(token: T) -> Self
  where
    T: ToString,
  {
    let inner = InnerClient {
      http: Http::new(token.to_string()),
    };

    #[cfg(feature = "autoposter")]
    let inner = Arc::new(inner);

    Self { inner }
  }

  /// Fetches a user from a Discord ID.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested user does not exist ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   let user = client.get_user(661200758510977084).await.unwrap();
  ///
  ///   assert_eq!(user.username, "null");
  ///   assert_eq!(user.id, 661200758510977084);
  ///
  ///   println!("{:?}", user);
  /// }
  /// ```
  pub async fn get_user<I>(&self, id: I) -> Result<User>
  where
    I: SnowflakeLike,
  {
    let path = format!("/users/{}", id.as_snowflake());

    self.inner.http.request(GET, &path, None).await
  }

  /// Fetches a listed Discord bot from a Discord ID.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested Discord bot is not listed on [Top.gg](https://top.gg) ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   let bot = client.get_bot(264811613708746752).await.unwrap();
  ///
  ///   assert_eq!(bot.username, "Luca");
  ///   assert_eq!(bot.id, 264811613708746752);
  ///
  ///   println!("{:?}", bot);
  /// }
  /// ```
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}", id.as_snowflake());

    self.inner.http.request(GET, &path, None).await
  }

  /// Fetches your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested Discord bot is not listed on [Top.gg](https://top.gg) ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   let stats = client.get_stats().await.unwrap();
  ///
  ///   println!("{:?}", stats);
  /// }
  /// ```
  #[inline(always)]
  pub async fn get_stats(&self) -> Result<Stats> {
    self.inner.http.request(GET, "/bots/stats", None).await
  }

  /// Posts your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Client, NewStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   let server_count = 1234; // be TRUTHFUL!
  ///   let shard_count = 10;
  ///
  ///   let stats = NewStats::count_based(server_count, Some(shard_count));
  ///
  ///   client.post_stats(stats).await.unwrap();
  /// }
  /// ```
  #[inline(always)]
  pub async fn post_stats(&self, new_stats: NewStats) -> Result<()> {
    self.inner.post_stats(&new_stats).await
  }

  /// Creates a new autoposter instance for this client which lets you automate the process of posting your Discord bot's statistics to the [Top.gg](https://top.gg) API in intervals.
  ///
  /// # Panics
  ///
  /// Panics if the interval argument is shorter than 15 minutes (900 seconds)
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use core::time::Duration;
  /// use topgg::{Autoposter, Client, NewStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   // creates an autoposter that posts data to Top.gg every 1800 seconds (15 minutes).
  ///   // the autopost thread will stop once it's dropped.
  ///   let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  ///   // ... then in some on ready/new guild event ...
  ///   let server_count = 12345;
  ///   let stats = NewStats::count_based(server_count, None);
  ///   autoposter.feed(stats).await;
  /// }
  /// ```
  #[cfg(feature = "autoposter")]
  #[cfg_attr(docsrs, doc(cfg(feature = "autoposter")))]
  pub fn new_autoposter(&self, interval: Duration) -> Autoposter {
    assert!(
      interval.as_secs() >= 900,
      "the interval mustn't be shorter than 15 minutes."
    );

    Autoposter::new(Arc::clone(&self.inner), interval)
  }

  /// Fetches your Discord bot's last 1000 voters.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   for voter in client.get_voters().await.unwrap() {
  ///     println!("{:?}", voter);
  ///   }
  /// }
  /// ```
  #[inline(always)]
  pub async fn get_voters(&self) -> Result<Vec<Voter>> {
    self.inner.http.request(GET, "/bots/votes", None).await
  }

  /// Queries/searches through the [Top.gg](https://top.gg) database to look for matching listed Discord bots.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested Discord bot is not listed on [Top.gg](https://top.gg) ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Client, Filter, Query};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   // inputting a string searches a bot that matches that username.
  ///   for bot in client.get_bots("shiro").await.unwrap() {
  ///     println!("{:?}", bot);
  ///   }
  ///
  ///   // advanced query with filters...
  ///   let filter = Filter::new()
  ///     .username("shiro")
  ///     .certified(true);
  ///
  ///   let query = Query::new()
  ///     .limit(250)
  ///     .skip(50)
  ///     .filter(filter);
  ///
  ///   for bot in client.get_bots(query).await.unwrap() {
  ///     println!("{:?}", bot);
  ///   }
  /// }
  /// ```
  pub async fn get_bots<Q>(&self, query: Q) -> Result<Vec<Bot>>
  where
    Q: QueryLike,
  {
    let path = format!("/bots{}", query.into_query_string());

    self
      .inner
      .http
      .request::<Bots>(GET, &path, None)
      .await
      .map(|res| res.results)
  }

  /// Checks if the specified user has voted for your Discord bot.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The user ID argument is a string and it's not a valid ID (expected things like `"123456789"`)
  /// - The client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   if client.has_voted(661200758510977084).await.unwrap() {
  ///     println!("checks out");
  ///   }
  /// }
  /// ```
  #[allow(clippy::transmute_int_to_bool)]
  pub async fn has_voted<I>(&self, user_id: I) -> Result<bool>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/votes?userId={}", user_id.as_snowflake());

    self
      .inner
      .http
      .request(GET, &path, None)
      .await
      .map(|res: Voted| res.voted != 0)
  }

  /// Checks if the weekend multiplier is active.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  ///   /// - HTTP or JSON serialization fail
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN"));
  ///
  ///   if client.is_weekend().await.unwrap() {
  ///     println!("guess what? it's the weekend! woohoo! 🎉🎉🎉🎉");
  ///   }
  /// }
  /// ```
  #[inline(always)]
  pub async fn is_weekend(&self) -> Result<bool> {
    self
      .inner
      .http
      .request(GET, "/weekend", None)
      .await
      .map(|res: IsWeekend| res.is_weekend)
  }
}
