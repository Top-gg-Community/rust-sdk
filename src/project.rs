use super::snowflake;

use serde::{Deserialize, Serialize};

/// A project's platform.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
  Discord,
}

/// A project's type.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
  Bot,
  Server,
}

/// A brief information on a project listed on Top.gg.
#[cfg(feature = "webhooks")]
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub struct PartialProject {
  #[serde(deserialize_with = "snowflake::deserialize")]
  /// The project's ID.
  pub id: u64,

  /// The project's type.
  #[serde(rename = "type")]
  pub kind: ProjectType,

  /// The project's platform.
  pub platform: Platform,

  /// The project's platform ID.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub platform_id: u64,
}

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    use serde::{de::DeserializeOwned, ser::Error};

    /// A project listed on Top.gg.
    #[derive(Clone, Debug, Deserialize)]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub struct Project {
      /// The project's ID.
      #[serde(deserialize_with = "snowflake::deserialize")]
      pub id: u64,

      /// The project's name sourced from the external platform.
      pub name: String,

      /// The project's platform.
      pub platform: Platform,

      /// The project's type.
      #[serde(rename = "type")]
      pub kind: ProjectType,

      /// The project's short description.
      pub headline: String,

      /// The project's tag IDs.
      pub tags: Vec<String>,

      /// The project's current vote count that affects the project's ranking.
      #[serde(rename = "votes")]
      pub current_votes: u64,

      /// The project's total vote count.
      #[serde(rename = "votes_total")]
      pub total_votes: u64,

      /// The project's review score out of 5.
      pub review_score: f32,

      /// The project's total review count.
      pub review_count: u64,
    }

    /// Retrieves an array of application commands in [Discord API's raw JSON format](https://discord.com/developers/docs/interactions/application-commands#application-command-object). Intended for use in [`Client::post_commands`][super::Client::post_commands].
    #[async_trait::async_trait]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub trait GetCommands<C, E>
    where
      C: Serialize + DeserializeOwned,
    {
      async fn get_commands(self) -> Result<Vec<C>, E>;
    }

    #[async_trait::async_trait]
    impl GetCommands<Self, serde_json::Error> for serde_json::Value {
      async fn get_commands(self) -> Result<Vec<Self>, serde_json::Error> {
        match self {
          Self::Array(elements) => Ok(elements),

          _ => Err(serde_json::Error::custom(
            "The object passed to get_commands() must be an array of commands.",
          )),
        }
      }
    }

    #[async_trait::async_trait]
    impl<C> GetCommands<C, ()> for Vec<C>
    where
      C: Serialize + DeserializeOwned + Send,
    {
      async fn get_commands(self) -> Result<Self, ()> {
        Ok(self)
      }
    }

    cfg_if::cfg_if! {
      if #[cfg(feature = "serenity")] {
        use serenity::{
          client::Context as SerenityContext,
          http::{
            HttpError as SerenityHttpError, LightMethod as SerenityHttpMethod,
            Request as SerenityHttpRequest, Route as SerenityHttpRoute,
          },
          Error as SerenityError,
        };

        #[async_trait::async_trait]
        #[cfg_attr(docsrs, doc(cfg(all(feature = "api", feature = "serenity"))))]
        impl GetCommands<serde_json::Value, SerenityError> for &SerenityContext {
          async fn get_commands(self) -> Result<Vec<serde_json::Value>, SerenityError> {
            let Some(application_id) = self.http.application_id() else {
              return Err(SerenityHttpError::ApplicationIdMissing.into());
            };

            self
              .http
              .fire::<_>(SerenityHttpRequest::new(
                SerenityHttpRoute::Commands { application_id },
                SerenityHttpMethod::Get,
              ))
              .await
          }
        }
      }
    }

    cfg_if::cfg_if! {
      if #[cfg(feature = "twilight")] {
        use twilight_http::{error::Error as TwilightHttpError, response::DeserializeBodyError as TwilightHttpDeserializeBodyError, request::application::command::GetGlobalCommands as TwilightGetGlobalCommands};
        use twilight_model::application::command::Command as TwilightCommand;

        #[doc(hidden)]
        #[derive(Debug)]
        pub enum TwilightGetCommandsError {
          Http(TwilightHttpError),
          Deserialize(TwilightHttpDeserializeBodyError),
        }

        #[async_trait::async_trait]
        #[cfg_attr(docsrs, doc(cfg(all(feature = "api", feature = "twilight"))))]
        impl GetCommands<TwilightCommand, TwilightGetCommandsError> for TwilightGetGlobalCommands<'_> {
          async fn get_commands(self) -> Result<Vec<TwilightCommand>, TwilightGetCommandsError> {
            self.await.map_err(TwilightGetCommandsError::Http)?.models().await.map_err(TwilightGetCommandsError::Deserialize)
          }
        }
      }
    }
  }
}
