use crate::{
  bot::{Bot, Bots, GetBots, IsWeekend},
  user::{User, Voted, Voter},
  util, Error, Result, Snowflake, Stats,
};
use reqwest::{header, IntoUrl, Method, Response, StatusCode, Version};
use serde::{de::DeserializeOwned, Deserialize};

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    use crate::autoposter;
    use std::sync::Arc;

    type SyncedClient = Arc<InnerClient>;
  } else {
    type SyncedClient = InnerClient;
  }
}

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
struct Ratelimit {
  retry_after: u16,
}

macro_rules! api {
  ($e:literal) => {
    concat!("https://top.gg/api", $e)
  };

  ($e:literal, $($rest:tt)*) => {
    format!(api!($e), $($rest)*)
  };
}

pub struct InnerClient {
  http: reqwest::Client,
  token: String,
  id: u64,
}

// This is implemented here because autoposter needs to access this struct from a different thread.
impl InnerClient {
  pub(crate) fn new(mut token: String) -> Self {
    let id = util::parse_api_token(&token);

    token.insert_str(0, "Bearer ");

    Self {
      http: reqwest::Client::new(),
      token,
      id,
    }
  }

  async fn send_inner(&self, method: Method, url: impl IntoUrl, body: Vec<u8>) -> Result<Response> {
    match self
      .http
      .execute(
        self
          .http
          .request(method, url)
          .header(header::AUTHORIZATION, &self.token)
          .header(header::CONNECTION, "close")
          .header(header::CONTENT_LENGTH, body.len())
          .header(header::CONTENT_TYPE, "application/json")
          .header(
            header::USER_AGENT,
            "topgg (https://github.com/top-gg/rust-sdk) Rust",
          )
          .version(Version::HTTP_11)
          .body(body)
          .build()
          .unwrap(),
      )
      .await
    {
      Ok(response) => {
        let status = response.status();

        if status.is_success() {
          Ok(response)
        } else {
          Err(match status {
            StatusCode::BAD_REQUEST => Error::InvalidRequest,
            StatusCode::UNAUTHORIZED => panic!("Invalid Top.gg API token."),
            StatusCode::NOT_FOUND => Error::NotFound,
            StatusCode::TOO_MANY_REQUESTS => match util::parse_json::<Ratelimit>(response).await {
              Ok(ratelimit) => Error::Ratelimit {
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
  pub(crate) async fn send<T>(
    &self,
    method: Method,
    url: impl IntoUrl,
    body: Option<Vec<u8>>,
  ) -> Result<T>
  where
    T: DeserializeOwned,
  {
    match self.send_inner(method, url, body.unwrap_or_default()).await {
      Ok(response) => util::parse_json(response).await,
      Err(err) => Err(err),
    }
  }

  pub(crate) async fn post_stats(&self, new_stats: &Stats) -> Result<()> {
    if new_stats.server_count.unwrap_or_default() == 0 {
      return Ok(());
    }

    self
      .send_inner(
        Method::POST,
        api!("/bots/stats"),
        serde_json::to_vec(new_stats).unwrap(),
      )
      .await
      .map(|_| ())
  }
}

/// Interact with the API's endpoints.
#[must_use]
pub struct Client {
  inner: SyncedClient,
}

impl Client {
  /// Creates a new instance.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Example
  ///
  /// ```rust
  /// let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// ```
  #[inline(always)]
  pub fn new(token: String) -> Self {
    let inner = InnerClient::new(token);

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
  /// - The client uses an invalid Top.gg API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to Top.gg ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the Top.gg servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The requested user does not exist ([`NotFound`][crate::Error::NotFound])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,should_panic
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let user = client.get_user(661200758510977084).await.unwrap();
  /// # }
  /// ```
  #[allow(clippy::unused_async)]
  #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
  pub async fn get_user<I>(&self, _id: I) -> Result<User>
  where
    I: Snowflake,
  {
    Err(Error::NotFound)
  }

  /// Fetches a Discord bot from its ID.
  ///
  /// # Panics
  ///
  /// Panics if:
  /// - The specified ID is invalid.
  /// - The client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - The specified bot does not exist. ([`NotFound`][crate::Error::NotFound])
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let bot = client.get_bot(264811613708746752).await.unwrap();
  /// # }
  /// ```
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: Snowflake,
  {
    self
      .inner
      .send(Method::GET, api!("/bots/{}", id.as_snowflake()), None)
      .await
  }

  /// Fetches your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let stats = client.get_stats().await.unwrap();
  /// # }
  /// ```
  pub async fn get_stats(&self) -> Result<Stats> {
    self
      .inner
      .send(Method::GET, api!("/bots/stats"), None)
      .await
  }

  /// Updates your Discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - The bot is in zero servers. ([`InvalidRequest`][crate::Error::InvalidRequest])
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// use topgg::Stats;
  /// #
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// //                            Server count
  /// client.post_stats(Stats::from(2)).await.unwrap();
  /// # }
  /// ```
  #[inline(always)]
  pub async fn post_stats(&self, new_stats: Stats) -> Result<()> {
    self.inner.post_stats(&new_stats).await
  }

  /// Fetches your project's recent unique voters.
  ///
  /// The amount of voters returned can't exceed 100, so you would need to use the `page` argument for this.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// //                             Page number
  /// let voters = client.get_voters(None).await.unwrap();
  ///
  /// for voter in voters {
  ///   println!("{}", voter.username);
  /// }
  /// # }
  /// ```
  pub async fn get_voters(&self, page: Option<usize>) -> Result<Vec<Voter>> {
    let page = match page {
      Some(page) => {
        if page >= 1 {
          page
        } else {
          1
        }
      }
      None => 1,
    };

    self
      .inner
      .send(
        Method::GET,
        api!("/bots/{}/votes?page={}", self.inner.id, page),
        None,
      )
      .await
  }

  pub(crate) async fn get_bots_inner(&self, query: String) -> Result<Vec<Bot>> {
    self
      .inner
      .send::<Bots>(Method::GET, api!("/bots{}", query), None)
      .await
      .map(|res| res.results)
  }

  /// Queries/searches through the Top.gg database to look for matching listed Discord bots.
  ///
  /// # Panics
  ///
  /// Panics if any of the client uses an invalid Top.gg API token (unauthorized).
  ///
  /// # Errors
  ///
  /// Errors if any of the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to Top.gg ([`InternalClientError`][crate::Error::InternalClientError])
  /// - An unexpected response from the Top.gg servers ([`InternalServerError`][crate::Error::InternalServerError])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// Basic usage:
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let bots = client
  ///   .get_bots()
  ///   .limit(250)
  ///   .skip(50)
  ///   .sort_by_monthly_votes()
  ///   .await
  ///   .unwrap();
  ///
  /// for bot in bots {
  ///   println!("{:?}", bot);
  /// }
  /// # }
  /// ```
  #[inline(always)]
  pub fn get_bots(&self) -> GetBots<'_> {
    GetBots::new(self)
  }

  /// Checks if a Top.gg user has voted for your project in the past 12 hours.
  ///
  /// # Panics
  ///
  /// Panics if:
  /// - The specified ID is invalid.
  /// - The client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - The specified user has not logged in to Top.gg. ([`NotFound`][crate::Error::NotFound])
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let has_voted = client.has_voted(8226924471638491136).await.unwrap();
  /// # }
  /// ```
  pub async fn has_voted<I>(&self, user_id: I) -> Result<bool>
  where
    I: Snowflake,
  {
    self
      .inner
      .send::<Voted>(
        Method::GET,
        api!("/bots/check?userId={}", user_id.as_snowflake()),
        None,
      )
      .await
      .map(|res| res.voted != 0)
  }

  /// Checks if the weekend multiplier is active, where a single vote counts as two.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust
  /// # #[tokio::main]
  /// # async fn main() {
  /// # let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// #
  /// let is_weekend = client.is_weekend().await.unwrap();
  /// # }
  /// ```
  pub async fn is_weekend(&self) -> Result<bool> {
    self
      .inner
      .send::<IsWeekend>(Method::GET, api!("/weekend"), None)
      .await
      .map(|res| res.is_weekend)
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "autoposter")] {
    impl autoposter::AsClientSealed for Client {
      #[inline(always)]
      fn as_client(&self) -> Arc<InnerClient> {
        Arc::clone(&self.inner)
      }
    }

    impl autoposter::AsClient for Client {}
  }
}
