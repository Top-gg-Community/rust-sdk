use core::{fmt, result};
use std::{error, io};
use tokio_native_tls::native_tls;

/// A struct representing an unexpected internal error coming from the client itself, preventing it from sending a request to the [top.gg](https://top.gg) API.
#[derive(Debug)]
pub enum InternalError {
  /// The client couldn't create a TLS connector.
  CreateConnector(native_tls::Error),

  /// The client connect to [top.gg](https://top.gg)'s servers.
  Connect(io::Error),

  /// The client couldn't establish a handshake with [top.gg](https://top.gg)'s servers.
  Handshake(native_tls::Error),

  /// The client couldn't write a HTTP request to [top.gg](https://top.gg)'s servers.
  WriteRequest(io::Error),
}

impl fmt::Display for InternalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::CreateConnector(_) => write!(f, "can't initiate a TLS connector"),
      Self::Connect(_) => write!(f, "can't initiate a TCP stream to top.gg"),
      Self::Handshake(_) => write!(f, "can't initiate a handshake with top.gg"),
      Self::WriteRequest(_) => write!(f, "can't write a request to top.gg"),
    }?;

    // look several lines below - it always returns Some()!
    write!(f, " (original error: {})", unsafe {
      error::Error::source(self).unwrap_unchecked()
    })
  }
}

impl error::Error for InternalError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    Some(match self {
      Self::CreateConnector(err) => err,
      Self::Connect(err) => err,
      Self::Handshake(err) => err,
      Self::WriteRequest(err) => err,
    })
  }
}

/// A struct representing an error coming from this SDK - unexpected or not.
#[derive(Debug)]
pub enum Error {
  /// An unexpected internal error coming from the client itself, preventing it from sending a request to the [top.gg](https://top.gg) API.
  InternalClientError(InternalError),

  /// An unexpected error coming from [top.gg](https://top.gg)'s servers themselves.
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
