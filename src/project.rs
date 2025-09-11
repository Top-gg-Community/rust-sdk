use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A project's reviews on Top.gg.
#[must_use]
#[derive(Clone, Debug, Deserialize)]
pub struct Reviews {
  /// This project's average review score out of 5.
  #[serde(rename = "averageScore")]
  pub score: f64,

  /// This project's review count.
  pub count: usize,
}

/// Retrieves an array of application commands in [Discord API's raw JSON format](https://discord.com/developers/docs/interactions/application-commands#application-command-object). For use in [`Client::post_bot_commands`][crate::Client::post_bot_commands].
#[async_trait::async_trait]
pub trait GetBotCommands<C, E>
where
  C: Serialize + DeserializeOwned,
{
  async fn get_bot_commands(self) -> Result<Vec<C>, E>;
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "serenity", feature = "serenity-cached"))] {
    use serenity::{
      client::Context as SerenityContext,
      http::{
        HttpError as SerenityHttpError, LightMethod as SerenityHttpMethod,
        Request as SerenityHttpRequest, Route as SerenityHttpRoute,
      },
      Error as SerenityError,
    };

    #[async_trait::async_trait]
    impl GetBotCommands<serde_json::Value, SerenityError> for &SerenityContext {
      async fn get_bot_commands(self) -> Result<Vec<serde_json::Value>, SerenityError> {
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
  if #[cfg(any(feature = "twilight", feature = "twilight-cached"))] {
    use twilight_http::{error::Error as TwilightHttpError, response::DeserializeBodyError as TwilightHttpDeserializeBodyError, request::application::command::GetGlobalCommands as TwilightGetGlobalCommands};
    use twilight_model::application::command::Command as TwilightCommand;

    #[doc(hidden)]
    #[derive(Debug)]
    pub enum TwilightGetCommandsError {
      Http(TwilightHttpError),
      Deserialize(TwilightHttpDeserializeBodyError),
    }

    #[async_trait::async_trait]
    impl GetBotCommands<TwilightCommand, TwilightGetCommandsError> for TwilightGetGlobalCommands<'_> {
      async fn get_bot_commands(self) -> Result<Vec<TwilightCommand>, TwilightGetCommandsError> {
        self.await.map_err(TwilightGetCommandsError::Http)?.models().await.map_err(TwilightGetCommandsError::Deserialize)
      }
    }
  }
}
