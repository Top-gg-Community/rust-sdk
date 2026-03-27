use super::{
  Error, GetCommands, PaginatedVotes, PaginatedVotesOwned, PartialVote, PostCommandsError,
  PostCommandsResult, Project, Result, Snowflake, UserSource, util,
};

use chrono::{DateTime, SecondsFormat, TimeZone};
use reqwest::{IntoUrl, Method, Response, StatusCode, Version, header};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[macro_export]
macro_rules! api {
  ($e:literal) => {
    concat!("https://top.gg/api/v1", $e)
  };

  ($e:literal, $($rest:tt)*) => {
    format!($crate::client::api!($e), $($rest)*)
  };
}

pub(super) use api;

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
struct Ratelimit {
  retry_after: u16,
}

/// Interact with Top.gg API v1's endpoints.
#[must_use]
pub struct Client {
  http: reqwest::Client,
  token: String,
}

impl Client {
  /// Creates a new client instance.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// let client = topgg::Client::new(env!("TOPGG_TOKEN").into());
  /// ```
  pub fn new(token: String) -> Self {
    Self {
      http: reqwest::Client::new(),
      token: format!("Bearer {token}"),
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
            StatusCode::UNAUTHORIZED => panic!("Invalid API token."),

            StatusCode::FORBIDDEN => Error::Forbidden,

            StatusCode::NOT_FOUND => Error::NotFound,

            StatusCode::TOO_MANY_REQUESTS => util::parse_json::<Ratelimit>(response).await.map_or(
              Error::InternalServerError,
              |ratelimit| Error::Ratelimit {
                retry_after: ratelimit.retry_after,
              },
            ),

            _ => Error::InternalServerError,
          })
        }
      }

      Err(err) => Err(Error::InternalClientError(err)),
    }
  }

  async fn send<T>(&self, method: Method, url: impl IntoUrl, body: Option<Vec<u8>>) -> Result<T>
  where
    T: DeserializeOwned,
  {
    match self.send_inner(method, url, body.unwrap_or_default()).await {
      Ok(response) => util::parse_json(response).await,

      Err(err) => Err(err),
    }
  }

  /// Tries to get your project's information.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - HTTP request failure from the client-side. ([`InternalClientError`][super::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][super::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][super::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// let project = client.get_self().await.unwrap();
  /// ```
  pub async fn get_self(&self) -> Result<Project> {
    self.send(Method::GET, api!("/projects/@me"), None).await
  }

  /// Tries to update the application commands list in your Discord bot's Top.gg page.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - Unable to retrieve the list of bot commands. ([`PostCommandsError::Retrieval`][super::PostCommandsError::Retrieval])
  /// - Unable to serialize the list of bot commands. ([`PostCommandsError::Serialization`][super::PostCommandsError::Serialization])
  /// - The list of bot commands supplied do not match [Discord API's raw JSON format](https://discord.com/developers/docs/interactions/application-commands#application-command-object). ([`Error::InvalidRequest`][super::Error::InvalidRequest])
  /// - HTTP request failure from the client-side. ([`Error::InternalClientError`][super::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`Error::InternalServerError`][super::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Error::Ratelimit`][super::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// // Serenity:
  /// client.post_commands(&ctx).await.unwrap();
  ///
  /// // Twilight:
  /// let application_id = bot.current_user_application().await.unwrap().model().await.unwrap().id;
  /// let interaction = bot.interaction(application_id);
  ///
  /// client.post_commands(interaction.global_commands()).await.unwrap();
  ///
  /// // Others:
  /// let commands = json!([{
  ///   "id": "1",
  ///   "type": 1,
  ///   "application_id": "1",
  ///   "name": "test",
  ///   "description": "command description",
  ///   "default_member_permissions": "",
  ///   "version": "1"
  /// }]); // Array of application commands that
  ///      // can be serialized to Discord API's raw JSON format.
  ///
  /// client.post_commands(commands).await.unwrap();
  /// ```
  pub async fn post_commands<L, C, E>(&self, context: C) -> PostCommandsResult<(), E>
  where
    L: Serialize + DeserializeOwned,
    C: GetCommands<L, E>,
  {
    let commands = context
      .get_commands()
      .await
      .map_err(PostCommandsError::Retrieval)?;

    match self
      .send_inner(
        Method::POST,
        api!("/projects/@me/commands"),
        serde_json::to_vec(&commands).map_err(PostCommandsError::Serialization)?,
      )
      .await
    {
      Ok(_) => Ok(()),

      Err(err) => Err(PostCommandsError::Request(err)),
    }
  }

  /// Tries to get the latest vote information of a user on your project. Returns [`None`] if the user has not voted.
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
  /// - The specified user has not logged in to Top.gg. ([`NotFound`][super::Error::NotFound])
  /// - HTTP request failure from the client-side. ([`InternalClientError`][super::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][super::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][super::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use topgg::UserSource;
  ///
  /// // Discord ID:
  /// let vote = client.get_vote(UserSource::Discord(661200758510977084)).await.unwrap();
  ///
  /// // Top.gg ID:
  /// let vote = client.get_vote(UserSource::Topgg(8226924471638491136)).await.unwrap();
  /// ```
  pub async fn get_vote<S>(&self, user: UserSource<S>) -> Result<Option<PartialVote>>
  where
    S: Snowflake,
  {
    match self
      .send(
        Method::GET,
        api!(
          "/projects/@me/votes/{}?source={}",
          user.as_snowflake(),
          user.name()
        ),
        None,
      )
      .await
    {
      Ok(vote) => Ok(Some(vote)),

      Err(err) => {
        if matches!(err, Error::NotFound) {
          return Ok(None);
        }

        Err(err)
      }
    }
  }

  /// Tries to get a cursor-based paginated list of votes for your project, ordered by creation date.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid API token.
  ///
  /// # Errors
  ///
  /// Returns [`Err`] if:
  /// - HTTP request failure from the client-side. ([`InternalClientError`][super::Error::InternalClientError])
  /// - HTTP request failure from the server-side. ([`InternalServerError`][super::Error::InternalServerError])
  /// - Ratelimited from sending more requests. ([`Ratelimit`][super::Error::Ratelimit])
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use chrono::{TimeZone, Utc};
  ///
  /// let since = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();
  /// let first_page = client.get_votes(since).await.unwrap();
  ///
  /// for vote in first_page.iter() {
  ///   println!("{vote:?}");
  /// }
  ///
  /// let second_page = first_page.next().await.unwrap();
  ///
  /// for vote in second_page.iter() {
  ///   println!("{vote:?}");
  /// }
  /// ```
  pub async fn get_votes<Tz>(&self, since: DateTime<Tz>) -> Result<PaginatedVotes<'_>>
  where
    Tz: TimeZone,
  {
    self
      .send(
        Method::GET,
        api!(
          "/projects/@me/votes?startDate={}",
          urlencoding::encode(&since.to_rfc3339_opts(SecondsFormat::Millis, true))
        ),
        None,
      )
      .await
      .map(|data| PaginatedVotes { data, client: self })
  }

  pub(super) async fn get_next_votes(&self, cursor: &str) -> Result<PaginatedVotesOwned> {
    self
      .send(
        Method::GET,
        api!("/projects/@me/votes?cursor={}", cursor),
        None,
      )
      .await
  }
}
