use crate::{Error, Result};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};

pub(crate) const GET: Method = Method::Get;
pub(crate) const POST: Method = Method::Post;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Method {
  Get,
  Post,
}

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
pub(crate) struct Ratelimit {
  pub(crate) retry_after: u16,
}

#[derive(Clone)]
pub(crate) struct Http {
  token: String,
  client: reqwest::Client,
}

impl Http {
  pub(crate) fn new(token: String) -> Self {
    let client = reqwest::ClientBuilder::new()
      .user_agent(concat!(
        env!("CARGO_PKG_NAME"),
        "(",
        env!("CARGO_PKG_REPOSITORY"),
        ")",
        env!("CARGO_PKG_VERSION")
      ))
      .build()
      .unwrap();
    Self { token, client }
  }

  pub(crate) async fn send<'a>(
    &'a self,
    predicate: Method,
    path: &'a str,
    body: Option<String>,
  ) -> Result<String> {
    let endpoint = format!("https://top.gg/api{path}");
    let ready_request = match predicate {
      Method::Get => self.client.get(endpoint).bearer_auth(&self.token),
      Method::Post => self
        .client
        .post(endpoint)
        .bearer_auth(self.token.clone())
        .body(body.unwrap_or_else(|| "".to_string())),
    };
    let response = ready_request.send().await?;
    let status_code = response.status();
    if !response.status().is_success() {
      let err = match status_code {
        StatusCode::UNAUTHORIZED => Error::Unauthorized,
        StatusCode::NOT_FOUND => Error::NotFound,
        StatusCode::TOO_MANY_REQUESTS => Error::Ratelimit {
          retry_after: response.json::<Ratelimit>().await?.retry_after,
        },
        _ => Error::InternalServerError,
      };
      return Err(err);
    }
    Ok(response.text().await?)
  }

  pub(crate) async fn request<D>(
    &self,
    predicate: Method,
    path: &str,
    body: Option<String>,
  ) -> Result<D>
  where
    D: DeserializeOwned,
  {
    let data = self.send(predicate, path, body).await?;
    Ok(serde_json::from_str(&data)?)
  }
}
