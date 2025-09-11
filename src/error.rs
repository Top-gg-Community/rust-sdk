use serde_json::Error as SerdeJsonError;
use std::{error, fmt, result};

/// An error coming from this SDK.
#[derive(Debug)]
pub enum Error {
  /// HTTP request failure from the client-side.
  InternalClientError(reqwest::Error),

  /// HTTP request failure from the server-side.
  InternalServerError,

  /// Attempted to send an invalid request to the API.
  InvalidRequest,

  /// Such query does not exist. Inside is the message from the API if available.
  NotFound(Option<String>),

  /// Ratelimited from sending more requests.
  Ratelimit {
    /// How long the client should wait in seconds before it could send requests again without receiving a 429.
    retry_after: u16,
  },

  /// Endpoint is inaccessible with legacy API tokens.
  UnsupportedToken,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InternalClientError(err) => write!(f, "Internal Client Error: {err}"),
      Self::InternalServerError => write!(f, "Internal Server Error"),
      Self::InvalidRequest => write!(f, "Invalid Request"),
      Self::NotFound(message) => write!(
        f,
        "Not Found: {}",
        message.as_deref().unwrap_or("<no message>")
      ),
      Self::Ratelimit { retry_after } => write!(
        f,
        "Blocked by the API for an hour. Please try again in {retry_after} seconds",
      ),
      Self::UnsupportedToken => write!(f, "Endpoint is inaccessible with legacy API tokens"),
    }
  }
}

impl error::Error for Error {
  #[inline(always)]
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Self::InternalClientError(err) => err.source(),
      _ => None,
    }
  }
}

/// The result type primarily used in this SDK.
pub type Result<T> = result::Result<T, Error>;

/// An error coming from [`Client::post_bot_commands`][crate::Client::post_bot_commands].
#[derive(Debug)]
pub enum PostBotCommandsError<E> {
  /// Error happened while retrieving the bot commands in [`GetBotCommands`][crate::GetBotCommands].
  Retrieval(E),

  /// Error happened while serializing the bot commands.
  Serialization(SerdeJsonError),

  /// Error happened while sending the HTTP request.
  Request(Error),
}

impl<E> fmt::Display for PostBotCommandsError<E>
where
  E: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Retrieval(err) => write!(f, "Error while retrieving bot commands: {err:?}"),
      Self::Serialization(err) => write!(f, "Error while serializing bot commands: {err:?}"),
      Self::Request(err) => write!(f, "Error while posting bot commands: {err:?}"),
    }
  }
}

impl<E> error::Error for PostBotCommandsError<E>
where
  E: error::Error + 'static,
{
  #[inline(always)]
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Self::Retrieval(err) => Some(err),
      Self::Serialization(err) => Some(err),
      Self::Request(err) => err.source(),
    }
  }
}

/// The result type used in [`Client::post_bot_commands`][crate::Client::post_bot_commands].
pub type PostBotCommandsResult<T, E> = result::Result<T, PostBotCommandsError<E>>;
