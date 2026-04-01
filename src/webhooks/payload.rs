use super::super::{PartialProject, User, snowflake};
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use log::warn;
use serde::Deserialize;
use sha2::Sha256;

/// A webhook payload.
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub enum Payload {
  /// An `integration.create` webhook payload. Fires when a user has connected to your webhook integration.
  #[serde(rename = "integration.create")]
  IntegrationCreate {
    /// The unique identifier for this connection.
    #[serde(deserialize_with = "snowflake::deserialize")]
    connection_id: u64,

    /// The secret used to verify future webhook deliveries.
    #[serde(rename = "webhook_secret")]
    secret: String,

    /// The project that the integration refers to.
    project: PartialProject,

    /// The user who triggered this event.
    user: User,
  },

  /// An `integration.delete` webhook payload. Fires when a user has disconnected from your webhook integration.
  #[serde(rename = "integration.delete")]
  IntegrationDelete {
    /// The unique identifier for this connection.
    #[serde(deserialize_with = "snowflake::deserialize")]
    connection_id: u64,
  },

  /// A `webhook.test` webhook payload. Fires upon sent test from the project dashboard.
  #[serde(rename = "webhook.test")]
  Test {
    /// The project that the test refers to.
    project: PartialProject,

    /// The user who triggered this test.
    user: User,
  },

  /// A `vote.create` webhook payload. Fires when a user votes for your project.
  #[serde(rename = "vote.create")]
  VoteCreate {
    /// The vote's ID.
    #[serde(deserialize_with = "snowflake::deserialize")]
    id: u64,

    /// The number of votes this vote counted for. This is a rounded integer value which determines how many points this individual vote was worth.
    weight: u64,

    /// When the vote was cast.
    #[serde(rename = "created_at")]
    voted_at: DateTime<Utc>,

    /// When the vote expires and the user is required to vote again.
    expires_at: DateTime<Utc>,

    /// The project that received this vote.
    project: PartialProject,

    /// The user who voted for this project.
    user: User,
  },
}

impl Payload {
  #[allow(clippy::new_ret_no_self)]
  #[cfg(any(feature = "axum", feature = "warp"))]
  pub(super) fn new(
    now: DateTime<Utc>,
    body: String,
    signature: &str,
    secret: &str,
  ) -> PayloadResult {
    IncomingPayload::new(now, body, signature, "").map_or(PayloadResult::BadRequest, |incoming| {
      incoming.authenticate(secret)
    })
  }
}

/// A processed [`Payload`].
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub enum PayloadResult {
  /// The payload has been successfully authenticated.
  Accepted(Payload),

  /// The timestamp is outside of the accepted time window, possibly being a part of a replay attack.
  Forbidden,

  /// The request's headers are missing or invalid.
  BadRequest,

  /// The request's signature cannot be authenticated with the correct webhook secret.
  Unauthorized,

  /// Unable deserialize payload. This could possibly be a bug with the SDK.
  ///
  /// It's recommended to return a 200 and 204 status code and report this to the SDK's maintainers when this happens.
  DeserializationFailure,

  /// Unable to create a SHA-256 HMAC instance from the specified webhook secret.
  InternalServerError,
}

/// An incoming [`Payload`] that is yet to be [authenticated with a secret][IncomingPayload::authenticate].
///
/// # Examples
///
/// With actix-web:
///
/// ```rust,no_run
/// use topgg::{IncomingPayload, PayloadResult};
/// use std::io;
///
/// use actix_web::{
///   App, HttpServer,
///   error::{Error, ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
///   get, post,
/// };
///
/// #[get("/")]
/// async fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// // POST /webhook
/// #[post("/webhook")]
/// async fn webhook(payload: IncomingPayload) -> Result<&'static str, Error> {
///   match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
///     PayloadResult::Accepted(payload) => {
///       println!("{payload:?}");
///
///       Ok("ok")
///     }
///
///     PayloadResult::Forbidden => Err(ErrorForbidden("Forbidden")),
///
///     PayloadResult::BadRequest => Err(ErrorBadRequest("Bad Request")),
///
///     PayloadResult::Unauthorized => Err(ErrorUnauthorized("Unauthorized")),
///
///     PayloadResult::DeserializationFailure => Ok(""),
///
///     PayloadResult::InternalServerError => Err(ErrorInternalServerError("Internal Server Error")),
///   }
/// }
///
/// #[actix_web::main]
/// async fn main() -> io::Result<()> {
///   HttpServer::new(|| App::new().service(index).service(webhook))
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
///
/// With rocket:
///
/// ```rust,no_run
/// use topgg::{IncomingPayload, PayloadResult};
///
/// use rocket::{Build, Rocket, get, http::Status, launch, post, routes};
///
/// #[get("/")]
/// fn index() -> &'static str {
///   "Hello, World!"
/// }
///
/// // POST /webhook
/// #[post("/webhook", data = "<payload>")]
/// fn webhook(payload: IncomingPayload) -> Status {
///   match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
///     PayloadResult::Accepted(payload) => {
///       println!("{payload:?}");
///
///       Status::NoContent
///     }
///
///     PayloadResult::Forbidden => Status::Forbidden,
///
///     PayloadResult::BadRequest => Status::BadRequest,
///
///     PayloadResult::Unauthorized => Status::Unauthorized,
///
///     PayloadResult::DeserializationFailure => Status::NoContent,
///
///     PayloadResult::InternalServerError => Status::InternalServerError,
///   }
/// }
///
/// #[launch]
/// fn rocket() -> Rocket<Build> {
///   rocket::build().mount("/", routes![index, webhook])
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub struct IncomingPayload {
  timestamp: i64,
  now: i64,
  signature: Vec<u8>,
  body: String,
  trace: String,
}

impl IncomingPayload {
  /// Tries to create a new incoming payload from the current timestamp, a request body, an `x-topgg-signature` header, and an `x-topgg-trace` header. Returns [`None`] if the header values cannot be parsed.
  #[must_use]
  pub fn new(now: DateTime<Utc>, body: String, signature: &str, trace: &str) -> Option<Self> {
    let signature = signature
      .split(',')
      .filter_map(|p| p.split_once('='))
      .collect::<HashMap<_, _>>();

    if let (Some(timestamp), Some(signature)) = (signature.get("t"), signature.get("v1"))
      && let (Ok(timestamp), Ok(signature)) = (timestamp.parse(), hex::decode(signature))
    {
      Some(Self {
        timestamp,
        now: now.timestamp_millis(),
        signature,
        body,
        trace: trace.into(),
      })
    } else {
      None
    }
  }

  /// Tries to authenticate a valid secret with this request.
  #[must_use]
  pub fn authenticate(&self, secret: &str) -> PayloadResult {
    if (self.now - (self.timestamp * 1000)).abs() > 30000 {
      return PayloadResult::Forbidden;
    }

    let Ok(mut hmac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
      warn!(
        "Unable to create a SHA-256 HMAC instance from the specified webhook secret. Dismissing payload request."
      );

      return PayloadResult::InternalServerError;
    };

    hmac.update(format!("{}.{}", self.timestamp, self.body).as_bytes());

    if hmac.verify_slice(&self.signature).is_ok() {
      serde_json::from_str(&self.body).map_or_else(|_| {
        warn!(
          "Unable to parse Top.gg webhook payload. Please report this bug to the SDK maintainers.\n--- BEGIN BODY DUMP ---\n{}\n--- END BODY DUMP ---",
          self.body
        );

        PayloadResult::DeserializationFailure
      }, PayloadResult::Accepted)
    } else {
      PayloadResult::Unauthorized
    }
  }

  /// Retrieves the payload's `x-topgg-trace` header for debugging and correlating requests with Top.gg support.
  #[must_use]
  pub fn get_trace(&self) -> &str {
    &self.trace
  }
}
