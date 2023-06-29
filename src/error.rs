use std::{error::Error as StdError, num::NonZeroU16};

/// A struct representing an error coming from this SDK - unexpected or not.
#[derive(Debug)]
pub enum Error {
  /// An unexpected internal error coming from the client itself, preventing it from sending a request to the [Top.gg](https://top.gg) API.
  Http(reqwest::Error),

  /// JSON encoding or decoding failure
  Json(serde_json::Error),

  /// An unexpected error coming from [Top.gg](https://top.gg)'s servers themselves.
  InternalServerError,

  /// The requested resource does not exist. (404)
  NotFound,

  /// Your authorization is invalid. (401)
  Unauthorized,

  /// UnknownError
  UnknownHttpError(NonZeroU16),

  /// The client is being ratelimited from sending more HTTP requests.
  Ratelimit {
    /// The amount of seconds before the ratelimit is lifted.
    retry_after: u16,
  },
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Http(err) => write!(f, "reqwest error: {err}"),
      Self::Json(err) => write!(f, "json error: {err}"),
      Self::UnknownHttpError(c) => write!(f, "unknown http error error: {c}"),
      Self::InternalServerError => write!(f, "internal server error"),
      Self::Unauthorized => write!(f, "invalid authorization token"),
      Self::NotFound => write!(f, "not found"),
      Self::Ratelimit { retry_after } => write!(
        f,
        "this client is ratelimited, try again in {} seconds",
        retry_after / 60
      ),
    }
  }
}

impl StdError for Error {
  #[inline(always)]
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match self {
      Self::Http(err) => err.source(),
      _ => None,
    }
  }
}

impl From<reqwest::Error> for Error {
  fn from(value: reqwest::Error) -> Self {
    Self::Http(value)
  }
}

impl From<serde_json::Error> for Error {
  fn from(value: serde_json::Error) -> Self {
    Self::Json(value)
  }
}

/// The [`Result`][core::result::Result] type primarily used in this SDK.
pub type Result<T> = std::result::Result<T, Error>;
