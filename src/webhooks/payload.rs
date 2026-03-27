use super::super::{PartialProject, User, snowflake};

use chrono::{DateTime, Utc};
use serde::Deserialize;

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
  #[cfg(any(feature = "axum", feature = "warp"))]
  pub(super) fn new(signature: &str, body: &str, secret: &str) -> Option<Self> {
    use std::collections::HashMap;

    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let signature = signature
      .split(',')
      .filter_map(|p| p.split_once('='))
      .collect::<HashMap<_, _>>();

    let (Some(t), Some(signature)) = (signature.get("t"), signature.get("v1")) else {
      return None;
    };

    let mut hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;

    hmac.update(format!("{t}.{body}").as_bytes());

    let digest = hex::encode(hmac.finalize().into_bytes());

    if &digest == signature
      && let Ok(payload) = serde_json::from_str(body)
    {
      Some(payload)
    } else {
      None
    }
  }
}
