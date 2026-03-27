use std::{error, fmt, result};

use serde_json::Error as SerdeJsonError;

/// An error coming from the SDK.
#[derive(Debug)]
pub enum Error {
  /// HTTP request failure from the client-side.
  InternalClientError(reqwest::Error),

  /// HTTP request failure from the server-side.
  InternalServerError,

  /// Attempted to send an invalid request to the API.
  InvalidRequest,

  /// You don't have access to this endpoint.
  Forbidden,

  /// Such route does not exist.
  NotFound,

  /// Ratelimited from sending more requests.
  Ratelimit {
    /// How long the client should wait in seconds before it could send requests again without receiving a 429.
    retry_after: u16,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InternalClientError(err) => write!(f, "Internal Client Error: {err}"),

      Self::InternalServerError => write!(f, "Internal Server Error"),

      Self::InvalidRequest => write!(f, "Attempted to send an invalid request to the API"),

      Self::NotFound => write!(f, "Such route does not exist"),

      Self::Forbidden => write!(f, "You don't have access to this endpoint"),

      Self::Ratelimit { retry_after } => write!(
        f,
        "Blocked by the API for an hour. Please try again in {retry_after} seconds",
      ),
    }
  }
}

impl error::Error for Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Self::InternalClientError(err) => err.source(),

      _ => None,
    }
  }
}

/// An error coming from [`Client::post_commands`][super::Client::post_commands].
#[derive(Debug)]
pub enum PostCommandsError<E> {
  /// Error happened while retrieving the bot commands in [`GetCommands`][super::GetCommands].
  Retrieval(E),

  /// Error happened while serializing the bot commands.
  Serialization(SerdeJsonError),

  /// Error happened while sending the HTTP request.
  Request(Error),
}

impl<E> fmt::Display for PostCommandsError<E>
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

impl<E> error::Error for PostCommandsError<E>
where
  E: error::Error + 'static,
{
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Self::Retrieval(err) => Some(err),

      Self::Serialization(err) => Some(err),

      Self::Request(err) => err.source(),
    }
  }
}

/// The result type primarily used in this SDK.
pub type Result<T> = result::Result<T, Error>;

/// The result type used in [`Client::post_commands`][super::Client::post_commands].
pub type PostCommandsResult<T, E> = result::Result<T, PostCommandsError<E>>;
