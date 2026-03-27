use super::Error;

use reqwest::Response;
use serde::de::DeserializeOwned;

pub async fn parse_json<T>(response: Response) -> super::Result<T>
where
  T: DeserializeOwned,
{
  if let Ok(bytes) = response.bytes().await
    && let Ok(json) = serde_json::from_slice(&bytes)
  {
    return Ok(json);
  }

  Err(Error::InternalServerError)
}
