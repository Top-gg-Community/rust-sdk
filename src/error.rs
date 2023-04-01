use core::{fmt, result};
use std::{error, io};
use tokio_native_tls::native_tls;

#[derive(Debug)]
pub enum InternalError {
  CreateConnector(native_tls::Error),
  Connect(io::Error),
  Handshake(native_tls::Error),
  WriteRequest(io::Error),
}

impl fmt::Display for InternalError {
  #[inline(always)]
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

#[derive(Debug)]
pub enum Error {
  InternalClientError(InternalError),
  InternalServerError,
  InvalidArgument,
  NotFound,
  Ratelimited { retry_after: u16 },
}

impl fmt::Display for Error {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InternalClientError(err) => write!(f, "internal client error: {err}"),
      Self::InternalServerError => write!(f, "internal server error"),
      Self::InvalidArgument => write!(f, "invalid argument"),
      Self::NotFound => write!(f, "not found"),
      Self::Ratelimited { retry_after } => write!(
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

pub type Result<T> = result::Result<T, Error>;
