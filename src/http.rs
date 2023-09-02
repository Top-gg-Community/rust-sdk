use crate::{Error, InternalError, Result};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::TcpStream,
};
use tokio_native_tls::{native_tls, TlsConnector};

pub(crate) const GET: &str = "GET";
pub(crate) const POST: &str = "POST";

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
pub(crate) struct Ratelimit {
  pub(crate) retry_after: u16,
}

#[derive(Clone)]
pub(crate) struct Http {
  token: String,
}

impl Http {
  pub(crate) const fn new(token: String) -> Self {
    Self { token }
  }

  pub(crate) async fn send<'a>(
    &self,
    predicate: &'static str,
    path: &'a str,
    body: Option<&'a str>,
  ) -> Result<String> {
    let cx: TlsConnector = native_tls::TlsConnector::new()
      .map_err(|err| Error::InternalClientError(InternalError::CreateConnector(err)))?
      .into();

    let socket = TcpStream::connect("top.gg:443")
      .await
      .map_err(|err| Error::InternalClientError(InternalError::Connect(err)))?;

    let mut socket = cx
      .connect("top.gg", socket)
      .await
      .map_err(|err| Error::InternalClientError(InternalError::Handshake(err)))?;

    let body = body.unwrap_or_default();

    let payload = format!(
      "\
      {predicate} /api{path} HTTP/1.1\r\n\
      Authorization: Bearer {}\r\n\
      Connection: close\r\n\
      Content-Length: {}\r\n\
      Content-Type: application/json\r\n\
      Host: top.gg\r\n\
      User-Agent: topgg (https://github.com/top-gg/rust-sdk) Rust/\r\n\r\n{body}\
    ",
      self.token,
      body.len()
    );

    socket
      .write_all(payload.as_bytes())
      .await
      .map_err(|err| Error::InternalClientError(InternalError::WriteRequest(err)))?;

    let mut response = String::new();

    socket
      .read_to_string(&mut response)
      .await
      .map_err(|_| Error::InternalServerError)?;

    // we should never receive invalid raw HTTP responses - so unwrap_unchecked() is okay to use here
    let status_code: u16 = unsafe {
      response
        .split_ascii_whitespace()
        .nth(1)
        .unwrap_unchecked()
        .parse()
        .unwrap_unchecked()
    };

    if status_code >= 400 {
      Err(match status_code {
        401 => panic!("Invalid Top.gg API token."),
        404 => Error::NotFound,
        429 => Error::Ratelimit {
          retry_after: serde_json::from_str::<Ratelimit>(&response)
            .map_err(|_| Error::InternalServerError)?
            .retry_after,
        },
        _ => Error::InternalServerError,
      })
    } else {
      response.drain(unsafe { ..response.find("\r\n\r\n").unwrap_unchecked() + 4 });

      Ok(response)
    }
  }

  pub(crate) async fn request<D>(
    &self,
    predicate: &'static str,
    path: &str,
    body: Option<&str>,
  ) -> Result<D>
  where
    D: DeserializeOwned,
  {
    self
      .send(predicate, path, body)
      .await
      .and_then(|response| serde_json::from_str(&response).map_err(|_| Error::InternalServerError))
  }
}
