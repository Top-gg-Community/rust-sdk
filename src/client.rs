use crate::{
  bot::{Bot, Bots, IsWeekend, Stats},
  user::{User, Voted, Voter},
  util, Query, Result, Snowflake,
};
use reqwest::{IntoUrl, Method};
use serde::de::DeserializeOwned;

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    use crate::autoposter;
    use std::sync::Arc;

    type SyncedClient = Arc<InnerClient>;
  } else {
    type SyncedClient = InnerClient;
  }
}

pub struct InnerClient {
  http: reqwest::Client,
  token: String,
}

// this is implemented here because autoposter needs to access this struct from a different thread.
impl InnerClient {
  #[inline(always)]
  pub(crate) async fn send<T>(
    &self,
    method: Method,
    url: impl IntoUrl,
    body: Option<Vec<u8>>,
  ) -> Result<T>
  where
    T: DeserializeOwned,
  {
    match util::request(
      &self.http,
      &self.token,
      method,
      url,
      body.unwrap_or_default(),
    )
    .await
    {
      Ok(response) => util::parse_json(response).await,
      Err(err) => Err(err),
    }
  }

  #[inline(always)]
  pub(crate) async fn post_stats(&self, new_stats: &Stats) -> Result<()> {
    util::post_stats(&self.http, &self.token, new_stats).await
  }
}

/// A struct representing a [Top.gg API](https://docs.top.gg) client instance.
#[must_use]
pub struct Client {
  inner: SyncedClient,
}

impl Client {
  /// Creates a brand new client instance from a [Top.gg](https://top.gg) token.
  ///
  /// To get your [Top.gg](https://top.gg) token, [view this tutorial](https://github.com/top-gg/rust-sdk/assets/60427892/d2df5bd3-bc48-464c-b878-a04121727bff).
  #[inline(always)]
  pub fn new(mut token: String) -> Self {
    token.insert_str(0, "Bearer ");

    let inner = InnerClient {
      http: reqwest::Client::new(),
      token,
    };

    #[cfg(feature = "autoposter")]
    let inner = Arc::new(inner);

    Self { inner }
  }

  /// Fetches a user from a Discord ID.
  ///
  /// # Panics
  ///
  /// Panics if any of the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested user does not exist ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn get_user<I>(&self, id: I) -> Result<User>
  where
    I: Snowflake,
  {
    self
      .inner
      .send(
        Method::GET,
        util::api!("/users/{}", id.as_snowflake()),
        None,
      )
      .await
  }

  /// Fetches a listed Discord bot from a Discord ID.
  ///
  /// # Panics
  ///
  /// Panics if any of the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested Discord bot is not listed on [Top.gg](https://top.gg) ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: Snowflake,
  {
    self
      .inner
      .send(Method::GET, util::api!("/bots/{}", id.as_snowflake()), None)
      .await
  }

  /// Fetches your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn get_stats(&self) -> Result<Stats> {
    self
      .inner
      .send(Method::GET, util::api!("/bots/stats"), None)
      .await
  }

  /// Posts your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  #[inline(always)]
  pub async fn post_stats(&self, new_stats: Stats) -> Result<()> {
    self.inner.post_stats(&new_stats).await
  }

  /// Fetches your Discord bot's last 1000 voters.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn get_voters(&self) -> Result<Vec<Voter>> {
    self
      .inner
      .send(Method::GET, util::api!("/bots/votes"), None)
      .await
  }

  /// Queries/searches through the [Top.gg](https://top.gg) database to look for matching listed Discord bots.
  ///
  /// # Panics
  ///
  /// Panics if any of the following conditions are met:
  /// - The ID argument is a string but not numeric
  /// - The client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Client, Query};
  ///
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// // inputting a string searches a bot that matches that username.
  /// for bot in client.get_bots("shiro").await.unwrap() {
  ///   println!("{:?}", bot);
  /// }
  ///
  /// let query = Query::new()
  ///   .limit(250)
  ///   .skip(50)
  ///   .username("shiro")
  ///   .certified(true);
  ///
  /// for bot in client.get_bots(query).await.unwrap() {
  ///   println!("{:?}", bot);
  /// }
  /// ```
  pub async fn get_bots<Q>(&self, query: Q) -> Result<Vec<Bot>>
  where
    Q: Into<Query>,
  {
    self
      .inner
      .send::<Bots>(
        Method::GET,
        util::api!("/bots{}", query.into().query_string()),
        None,
      )
      .await
      .map(|res| res.results)
  }

  /// Checks if the specified user has voted for your Discord bot.
  ///
  /// # Panics
  ///
  /// Panics if any of the following conditions are met:
  /// - The user ID argument is a string and it's not a valid ID (expected things like `"123456789"`)
  /// - The client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn has_voted<I>(&self, user_id: I) -> Result<bool>
  where
    I: Snowflake,
  {
    self
      .inner
      .send::<Voted>(
        Method::GET,
        util::api!("/bots/votes?userId={}", user_id.as_snowflake()),
        None,
      )
      .await
      .map(|res| res.voted != 0)
  }

  /// Checks if the weekend multiplier is active.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [Top.gg API](https://docs.top.gg) token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the [Top.gg](https://top.gg) servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  pub async fn is_weekend(&self) -> Result<bool> {
    self
      .inner
      .send::<IsWeekend>(Method::GET, util::api!("/weekend"), None)
      .await
      .map(|res| res.is_weekend)
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    #[async_trait::async_trait]
    impl autoposter::IntoClientSealed for Client {
      type ArcInner = InnerClient;

      #[inline(always)]
      fn get_arc(&self) -> Arc<Self::ArcInner> {
        Arc::clone(&self.inner)
      }

      #[inline(always)]
      async fn post_stats(arc: &Self::ArcInner, stats: &Stats) {
        let _ = arc.post_stats(stats).await;
      }
    }

    impl autoposter::IntoClient for Client {}
  }
}
