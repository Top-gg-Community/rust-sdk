use crate::{
  bot::{Bot, BotQuery, Bots, IsWeekend, Stats},
  util,
  voter::{Voted, Voter},
  Error, Result, Snowflake,
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

#[macro_export]
macro_rules! api {
  ($e:literal) => {
    concat!("https://top.gg/api/v1", $e)
  };

  ($e:literal, $($rest:tt)*) => {
    format!($crate::client::api!($e), $($rest)*)
  };
}

pub(crate) use api;

pub struct InnerClient {
  http: reqwest::Client,
  token: String,
  id: u64,
}

// This is implemented here because autoposter needs to access this struct from a different thread.
impl InnerClient {
  pub(crate) fn new(token: String) -> Self {
    let id = util::id_from_token(&token);

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
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => panic!("Invalid API token."),
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

  pub(crate) async fn post_server_count(&self, server_count: usize) -> Result<()> {
    if server_count == 0 {
      return Err(Error::InvalidRequest);
    }

    self
      .send_inner(
        Method::POST,
        api!("/bots/stats"),
        serde_json::to_vec(&Stats {
          server_count: Some(server_count),
        })
        .unwrap(),
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
  /// To retrieve your API token, [see this tutorial](https://github.com/top-gg-community/rust-sdk/assets/60427892/d2df5bd3-bc48-464c-b878-a04121727bff).
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  /// ```
  #[inline(always)]
  pub fn new(token: String) -> Self {
    let inner = InnerClient::new(token);

    #[cfg(feature = "autoposter")]
    let inner = Arc::new(inner);

    Self { inner }
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
  /// ```rust,no_run
  /// let bot = client.get_bot(264811613708746752).await.unwrap();
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

  /// Fetches your Discord bot's posted server count.
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
  /// ```rust,no_run
  /// let server_count = client.get_server_count().await.unwrap();
  /// ```
  pub async fn get_server_count(&self) -> Result<Option<usize>> {
    self
      .inner
      .send(Method::GET, api!("/bots/stats"), None)
      .await
      .map(|stats: Stats| stats.server_count)
  }

  /// Updates the server count in your Discord bot's Top.gg page.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - The bot is currently in zero servers. ([`InvalidRequest`][crate::Error::InvalidRequest])
  /// - HTTP request failure from the client-side. ([`InternalClientError`][crate::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][crate::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][crate::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// client.post_server_count(bot.server_count()).await.unwrap();
  /// ```
  #[inline(always)]
  pub async fn post_server_count(&self, server_count: usize) -> Result<()> {
    self.inner.post_server_count(server_count).await
  }

  /// Fetches your Discord bot's recent unique voters.
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
  /// ```rust,no_run
  /// //                             Page number
  /// let voters = client.get_voters(1).await.unwrap();
  ///
  /// for voter in voters {
  ///   println!("{}", voter.username);
  /// }
  /// ```
  pub async fn get_voters(&self, mut page: usize) -> Result<Vec<Voter>> {
    if page < 1 {
      page = 1;
    }

    self
      .inner
      .send(
        Method::GET,
        api!("/bots/{}/votes?page={}", self.inner.id, page),
        None,
      )
      .await
  }

  pub(crate) async fn get_bots_inner(&self, path: String) -> Result<Vec<Bot>> {
    self
      .inner
      .send::<Bots>(Method::GET, api!("{}", path), None)
      .await
      .map(|res| res.results)
  }

  /// Fetches Discord bots that matches the specified query.
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
  /// ```rust,no_run
  /// let bots = client
  ///   .get_bots()
  ///   .limit(250)
  ///   .skip(50)
  ///   .sort_by_monthly_votes()
  ///   .await
  ///   .unwrap();
  ///
  /// for bot in bots {
  ///   println!("{}", bot.name);
  /// }
  /// ```
  #[inline(always)]
  pub fn get_bots(&self) -> BotQuery<'_> {
    BotQuery::new(self)
  }

  /// Checks if a Discord user has voted for your Discord bot in the past 12 hours.
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
  /// ```rust,no_run
  /// let has_voted = client.has_voted(661200758510977084).await.unwrap();
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
  /// ```rust,no_run
  /// let is_weekend = client.is_weekend().await.unwrap();
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
