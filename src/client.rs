use crate::{
  bot::{Bot, Bots, IsWeekend, Stats},
  user::{User, Voted, Voter},
  Error, Query, Result, Snowflake,
};
use core::{mem::transmute, str};
use reqwest::{RequestBuilder, Response, StatusCode, Version};
use serde::{de::DeserializeOwned, Deserialize};

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

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
pub(crate) struct Ratelimit {
  pub(crate) retry_after: u16,
}

macro_rules! api {
  ($e:expr) => {
    concat!("https://top.gg/api", $e)
  };

  ($e:expr,$($rest:tt)*) => {
    format!(api!($e), $($rest)*)
  };
}

pub(crate) struct InnerClient {
  http: reqwest::Client,
  token: String,
}

// this is implemented here because autoposter needs to access this function from a different thread.
impl InnerClient {
  async fn send_inner(&self, request: RequestBuilder, body: Vec<u8>) -> Result<Response> {
    let mut auth = String::with_capacity(self.token.len() + 7);
    auth.push_str("Bearer ");
    auth.push_str(&self.token);

    // SAFETY: the header keys and values should be valid ASCII
    match self
      .http
      .execute(unsafe {
        builder
          .header("Authorization", &auth)
          .header("Connection", "close")
          .header("Content-Length", body.len())
          .header("Content-Type", "application/json")
          .header(
            "User-Agent",
            "topgg (https://github.com/top-gg/rust-sdk) Rust",
          )
          .version(Version::HTTP_11)
          .body(body)
          .build()
          .unwrap_unchecked()
      })
      .await
    {
      Ok(response) => {
        let status = response.status();

        if status.is_success() {
          Ok(response)
        } else {
          Err(match status {
            StatusCode::UNAUTHORIZED => panic!("Invalid Top.gg API token."),
            StatusCode::NOT_FOUND => Error::NotFound,
            StatusCode::TOO_MANY_REQUESTS => match response.json() {
              Ok(Ratelimit { retry_after }) => Error::Ratelimit {
                retry_after: ratelimit.retry_after,
              },
              _ => Error::InternalServerError,
            },
            _ => Error::InternalServerError,
          })
        }
      }

      Err(err) => Err(Error::InternalClientError(err)),
    }
  }

  #[inline(always)]
  pub(crate) async fn send<T>(&self, request: RequestBuilder, body: Option<Vec<u8>>) -> Result<T>
  where
    T: DeserializeOwned,
  {
    self
      .send_inner(request, body.unwrap_or_default())
      .await
      .json()
      .map_err(|_| Error::InternalServerError)
  }

  pub(crate) async fn post_stats(&self, new_stats: &Stats) -> Result<()> {
    // SAFETY: no part of the Stats struct would cause an error in the serialization process.
    let body = unsafe { serde_json::to_vec(new_stats).unwrap_unchecked() };

    self
      .send_inner(self.http.post(api!("/bots/stats")), body)
      .await
      .map(|_| ())
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
  /// You can get a [Top.gg](https://top.gg) token if you own a listed Discord bot on [Top.gg](https://top.gg) (open the edit page, see in `Webhooks` section)
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// let _client = Client::new(env!("TOPGG_TOKEN").to_string());
  /// ```
  #[inline(always)]
  pub fn new(token: String) -> Self {
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  /// let user = client.get_user(661200758510977084).await.unwrap();
  ///
  /// assert_eq!(user.username, "null");
  /// assert_eq!(user.id, 661200758510977084);
  ///
  /// println!("{:?}", user);
  /// ```
  pub async fn get_user<I>(&self, id: I) -> Result<User>
  where
    I: Snowflake,
  {
    self
      .inner
      .send(
        self.inner.http.get(api!("/users/{}", id.as_snowflake())),
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  /// let bot = client.get_bot(264811613708746752).await.unwrap();
  ///
  /// assert_eq!(bot.username, "Luca");
  /// assert_eq!(bot.discriminator, "1375");
  /// assert_eq!(bot.id, 264811613708746752);
  ///
  /// println!("{:?}", bot);
  /// ```
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: Snowflake,
  {
    self
      .inner
      .send(
        self.inner.http.get(api!("/bots/{}", id.as_snowflake())),
        None,
      )
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  /// let stats = client.get_stats().await.unwrap();
  ///
  /// println!("{:?}", stats);
  /// ```
  #[inline(always)]
  pub async fn get_stats(&self) -> Result<Stats> {
    self
      .inner
      .send(self.inner.http.get(api!("/bots/stats")), None)
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// let server_count = 12345;
  /// client
  ///   .post_stats(Stats::count_based(server_count, None))
  ///   .await
  ///   .unwrap();
  /// ```
  #[inline(always)]
  pub async fn post_stats(&self, new_stats: Stats) -> Result<()> {
    self.inner.post_stats(&new_stats).await
  }

  /// Creates a new autoposter instance for this client which lets you automate the process of posting your Discord bot's statistics to the [Top.gg API](https://docs.top.gg) in intervals.
  ///
  /// # Panics
  ///
  /// Panics if any of the following conditions are met:
  /// - An autoposter thread is already running
  /// - The interval argument is shorter than 15 minutes (900 seconds)
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use core::time::Duration;
  /// use topgg::{Client, Stats};
  ///
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// // creates an autoposter that posts data to Top.gg every 1800 seconds (30 minutes).
  /// // the autopost thread will stop once it's dropped.
  /// let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  /// // ... then in some on ready/new guild event ...
  /// let server_count = 12345;
  /// let stats = Stats::count_based(server_count, None);
  /// autoposter.feed(stats).await;
  /// ```
  #[cfg(feature = "autoposter")]
  #[cfg_attr(docsrs, doc(cfg(feature = "autoposter")))]
  pub fn new_autoposter(&self, interval: Duration) -> Autoposter {
    assert!(
      Arc::strong_count(&self.inner) < 2,
      "An autoposter thread is already running."
    );
    assert!(
      interval.as_secs() >= 900,
      "The interval mustn't be shorter than 15 minutes."
    );

    Autoposter::new(Arc::clone(&self.inner), interval)
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// for voter in client.get_voters().await.unwrap() {
  ///   println!("{:?}", voter);
  /// }
  /// ```
  #[inline(always)]
  pub async fn get_voters(&self) -> Result<Vec<Voter>> {
    self
      .inner
      .send(self.inner.http.get(api!("/bots/votes")), None)
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
        self
          .inner
          .http
          .get(api!("/bots{}", query.into().query_string())),
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// if client.has_voted(661200758510977084).await.unwrap() {
  ///   println!("checks out");
  /// }
  /// ```
  #[allow(clippy::transmute_int_to_bool)]
  pub async fn has_voted<I>(&self, user_id: I) -> Result<bool>
  where
    I: Snowflake,
  {
    // SAFETY: res.voted will always be either 0 or 1.
    self
      .inner
      .send::<Voted>(
        self
          .inner
          .http
          .get("/bots/votes?userId={}", user_id.as_snowflake()),
        None,
      )
      .await
      .map(|res| unsafe { transmute(res.voted) })
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
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [Top.gg](https://top.gg) ([`InternalClientError`][crate::Error::InternalClientError])
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
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// if client.is_weekend().await.unwrap() {
  ///   println!("guess what? it's the weekend! woohoo! 🎉🎉🎉🎉");
  /// }
  /// ```
  #[inline(always)]
  pub async fn is_weekend(&self) -> Result<bool> {
    self
      .inner
      .send::<IsWeekend>(self.inner.http.get(api!("/weekend")), None)
      .await
      .map(|res| res.is_weekend)
  }
}
