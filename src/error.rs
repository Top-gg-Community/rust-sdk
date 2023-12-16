use core::{fmt, result};
use std::error;

/// A struct representing an error coming from this SDK - unexpected or not.
#[derive(Debug)]
pub enum Error {
  /// An unexpected internal error coming from the client itself, preventing it from sending a request to [Top.gg](https://top.gg).
  InternalClientError(reqwest::Error),

  /// An unexpected error coming from [Top.gg](https://top.gg)'s servers themselves.
  InternalServerError,

  /// The requested resource does not exist. (404)
  NotFound,

  /// The client is being ratelimited from sending more HTTP requests.
  Ratelimit {
    /// The amount of seconds before the ratelimit is lifted.
    retry_after: u16,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InternalClientError(err) => write!(f, "internal client error: {err}"),
      Self::InternalServerError => write!(f, "internal server error"),
      Self::NotFound => write!(f, "not found"),
      Self::Ratelimit { retry_after } => write!(
        f,
        "this client is ratelimited, try again in {} seconds",
        retry_after / 60
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

/// The [`Result`][core::result::Result] type primarily used in this SDK.
pub type Result<T> = result::Result<T, Error>;
