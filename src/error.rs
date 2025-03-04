use core::{fmt, result};
use std::error;

/// An error coming from this SDK.
#[derive(Debug)]
pub enum Error {
  /// An unexpected client-side error has occurred.
  InternalClientError(reqwest::Error),

  /// An unexpected server-side error has occurred.
  InternalServerError,

  /// Attempted to send an invalid request to the API.
  InvalidRequest,

  /// Such query does not exist.
  NotFound,

  /// Ratelimited from sending more requests.
  Ratelimit {
    /// How long the client should wait (in seconds) before it can make a request to the API again.
    retry_after: u16,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InternalClientError(err) => write!(f, "Internal Client Error: {err}"),
      Self::InternalServerError => write!(f, "Internal Server Error"),
      Self::InvalidRequest => write!(f, "Invalid Request"),
      Self::NotFound => write!(f, "Not Found"),
      Self::Ratelimit { retry_after } => write!(
        f,
        "Blocked by the API for an hour. Please try again in {retry_after} seconds",
      ),
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
