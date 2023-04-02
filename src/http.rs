use crate::{Error, InternalError, Result};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls, TlsConnector};

pub(crate) const GET: &str = "GET";
pub(crate) const POST: &str = "POST";

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
pub(crate) struct Ratelimit {
  pub(crate) retry_after: u16,
}

pub(crate) struct Http<'a> {
  token: &'a str,
}

impl<'a> Http<'a> {
  pub(crate) const fn new(token: &'a str) -> Self {
    Self { token }
  }

  async fn send(
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

    let payload = format!(
      "\
      {predicate} /api{path} HTTP/1.0\r\n\
      Authorization: Bearer {}\r\n\
      Content-Type: application/json\r\n\
      Host: top.gg\r\n\
      User-Agent: topgg (https://github.com/top-gg/rust-sdk) Rust/\r\n\
      \r\n{}\
    ",
      self.token,
      body.unwrap_or("")
    );

    if let Err(err) = socket.write_all(payload.as_bytes()).await {
      return Err(Error::InternalClientError(InternalError::WriteRequest(err)));
    }

    let mut response = String::new();

    if socket.read_to_string(&mut response).await.is_err() {
      return Err(Error::InternalServerError);
    }

    // we should never receive invalid raw HTTP responses - so unwrap_unchecked() is okay to use here
    let status_code = unsafe {
      response
        .split_ascii_whitespace()
        .nth(1)
        .unwrap_unchecked()
        .parse::<u16>()
        .unwrap_unchecked()
    };

    match status_code {
      401 => panic!("unauthorized"),
      404 => Err(Error::NotFound),
      429 => Err(Error::Ratelimit {
        retry_after: serde_json::from_str::<Ratelimit>(&response)
          .map_err(|_| Error::InternalServerError)?
          .retry_after,
      }),
      500.. => Err(Error::InternalServerError),
      _ => {
        response.drain(unsafe { ..response.find("\r\n\r\n").unwrap_unchecked() + 4 });

        Ok(response)
      }
    }
  }

  pub(crate) async fn request<D>(
    &self,
    predicate: &'static str,
    path: &'a str,
    body: Option<&'a str>,
  ) -> Result<D>
  where
    D: DeserializeOwned,
  {
    let response = self.send(predicate, path, body).await?;

    serde_json::from_str(&response).map_err(|_| Error::InternalServerError)
  }
}
