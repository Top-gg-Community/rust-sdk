use crate::{Error, Stats};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{header, IntoUrl, Method, Response, StatusCode, Version};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

const DISCORD_EPOCH: u64 = 1_420_070_400_000;

macro_rules! api {
  ($e:literal) => {
    concat!("https://top.gg/api", $e)
  };

  ($e:literal, $($rest:tt)*) => {
    format!(crate::util::api!($e), $($rest)*)
  };
}

pub(crate) use api;

#[inline(always)]
pub(crate) fn deserialize_optional_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(match <String as Deserialize>::deserialize(deserializer) {
    Ok(s) => {
      if s.is_empty() {
        None
      } else {
        Some(s)
      }
    }
    _ => None,
  })
}

#[inline(always)]
pub(crate) fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  T: Default + Deserialize<'de>,
  D: Deserializer<'de>,
{
  Option::deserialize(deserializer).map(|res| res.unwrap_or_default())
}

#[inline(always)]
pub(crate) fn get_creation_date(id: u64) -> DateTime<Utc> {
  Utc
    .timestamp_millis_opt(((id >> 22) + DISCORD_EPOCH) as _)
    .single()
    .unwrap()
}

#[inline(always)]
pub(crate) async fn parse_json<T>(response: Response) -> crate::Result<T>
where
  T: DeserializeOwned,
{
  if let Ok(bytes) = response.bytes().await {
    if let Ok(json) = serde_json::from_slice(&bytes) {
      return Ok(json);
    }
  }

  Err(Error::InternalServerError)
}

pub(crate) fn get_avatar(hash: &Option<String>, id: u64) -> String {
  match hash {
    Some(hash) => {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!("https://cdn.discordapp.com/avatars/{id}/{hash}.{ext}?size=1024")
    }
    _ => format!(
      "https://cdn.discordapp.com/embed/avatars/{}.png",
      (id >> 22) % 5
    ),
  }
}

#[derive(Deserialize)]
#[serde(rename = "kebab-case")]
struct Ratelimit {
  retry_after: u16,
}

pub(crate) async fn request(
  client: &reqwest::Client,
  token: &str,
  method: Method,
  url: impl IntoUrl,
  body: Vec<u8>,
) -> crate::Result<Response> {
  match client
    .execute(
      client
        .request(method, url)
        .header(header::AUTHORIZATION, token)
        .header(header::CONNECTION, "close")
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, "application/json")
        .header(
          header::USER_AGENT,
          "topgg (https://github.com/top-gg/rust-sdk) Rust",
        )
        .version(Version::HTTP_11)
        .body(body)
        .build()
        .unwrap(),
    )
    .await
  {
    Ok(response) => {
      let status = response.status();

      if status.is_success() {
        Ok(response)
      } else {
        Err(match status {
          StatusCode::UNAUTHORIZED => panic!("Invalid Top.gg API token."),
          StatusCode::NOT_FOUND => Error::NotFound,
          StatusCode::TOO_MANY_REQUESTS => match parse_json::<Ratelimit>(response).await {
            Ok(ratelimit) => Error::Ratelimit {
              retry_after: ratelimit.retry_after,
            },
            _ => Error::InternalServerError,
          },
          _ => Error::InternalServerError,
        })
      }
    }

    Err(err) => Err(Error::InternalClientError(err)),
  }
}

pub(crate) async fn post_stats(
  client: &reqwest::Client,
  token: &str,
  new_stats: &Stats,
) -> crate::Result<()> {
  request(
    client,
    token,
    Method::POST,
    api!("/bots/stats"),
    serde_json::to_vec(new_stats).unwrap(),
  )
  .await
  .map(|_| ())
}
