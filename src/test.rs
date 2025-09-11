use crate::{Client, UserSource};
use tokio::time::{sleep, Duration};

#[cfg(feature = "bot-autoposter")]
use crate::BotAutoposter;

macro_rules! delayed {
  ($($b:tt)*) => {
    $($b)*
    sleep(Duration::from_secs(1)).await
  };
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "serenity", feature = "serenity-cached", feature = "twilight", feature = "twilight-cached"))] {
    use crate::PostBotCommandsError;
    use std::sync::Arc;
    use tokio::sync::{mpsc, OnceCell};

    #[cfg(feature = "bot-autoposter")]
    use tokio::sync::Mutex;
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "serenity", feature = "serenity-cached"))] {
    use serenity::{
      Error as SerenityError,
      builder::CreateCommand as SerenityCreateCommand,
      client::{
        Client as SerenityClient,
        Context as SerenityContext,
        EventHandler as SerenityEventHandler
      },
      model::{application::Command as SerenityCommand, gateway::{Ready as SerenityReadyEvent, GatewayIntents as SerenityGatewayIntents}}
    };

    #[cfg(feature = "bot-autoposter")]
    use tokio::time::timeout;

    #[derive(Debug)]
    #[allow(dead_code)]
    enum SerenityTestError {
      PostBotCommandsSerenity(SerenityError),
      PostBotCommandsTopgg(PostBotCommandsError<SerenityError>),
      #[cfg(feature = "bot-autoposter")]
      BotAutoposterThread(crate::Error),
      #[cfg(feature = "bot-autoposter")]
      BotAutoposterTimeout,
    }

    struct SerenityTestEventHandler {
      client: Arc<Client>,
      result_sender: mpsc::Sender<Result<(), SerenityTestError>>,
      #[cfg(feature = "bot-autoposter")]
      bot_autoposter: Mutex<BotAutoposter<crate::SerenityBotAutoposter>>,
    }

    impl SerenityTestEventHandler {
      async fn test_post_bot_commands(&self, ctx: &SerenityContext) -> Result<(), SerenityTestError> {
        SerenityCommand::set_global_commands(&ctx.http, vec![SerenityCreateCommand::new("test").description("command description")]).await.map_err(SerenityTestError::PostBotCommandsSerenity)?;

        self.client.post_bot_commands(ctx).await.map_err(SerenityTestError::PostBotCommandsTopgg)
      }
    }

    static SERENITY_TEST_EVENT_HANDLER_READY_ONCE: OnceCell<()> = OnceCell::const_new();

    #[async_trait::async_trait]
    impl SerenityEventHandler for SerenityTestEventHandler {
      #[cfg_attr(not(feature = "bot-autoposter"), allow(unused_mut))]
      async fn ready(&self, ctx: SerenityContext, _ready: SerenityReadyEvent) {
        SERENITY_TEST_EVENT_HANDLER_READY_ONCE.get_or_init(|| async {
          let mut test_result = self.test_post_bot_commands(&ctx).await;

          #[cfg(feature = "bot-autoposter")]
          if test_result.is_ok() {
            let mut bot_autoposter_guard = self.bot_autoposter.lock().await;
            let mut bot_autoposter_receiver = bot_autoposter_guard.receiver();

            match timeout(Duration::from_secs(10), async move {
              let mut bot_autopost_counter = 0;

              while let Some(posted) = bot_autoposter_receiver.recv().await {
                if let Err(err) = posted {
                  return Err(err);
                }

                bot_autopost_counter += 1;

                if bot_autopost_counter == 3 {
                  break;
                }
              }

              Ok(())
            }).await {
              Ok(Err(err)) => test_result = Err(SerenityTestError::BotAutoposterThread(err)),
              Err(_) => test_result = Err(SerenityTestError::BotAutoposterTimeout),
              _ => {},
            }
          }

          self.result_sender.send(test_result).await.unwrap();
        }).await;
      }
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "twilight", feature = "twilight-cached"))] {
    use std::sync::atomic::{self, AtomicBool};
    use tokio::time::timeout;
    use twilight_gateway::{Event as TwilightGatewayEvent, Intents as TwilightGatewayIntents, Shard as TwilightShard, ShardId as TwilightShardId};
    use twilight_http::{Error as TwilightHttpError, response::DeserializeBodyError as TwilightHttpDeserializeBodyError, Client as TwilightHttpClient};
    use twilight_model::{application::command::{Command as TwilightCommand, CommandType as TwilightCommandType}, id::Id as TwilightId};

    static TWILIGHT_TEST_EVENT_HANDLER_READY_ONCE: OnceCell<()> = OnceCell::const_new();

    #[derive(Debug)]
    #[allow(dead_code)]
    enum TwilightTestError {
      PostBotCommandsTwilightApplicationIdHttp(TwilightHttpError),
      PostBotCommandsTwilightApplicationIdDeserialization(TwilightHttpDeserializeBodyError),
      PostBotCommandsTwilightSetBotCommands(TwilightHttpError),
      PostBotCommandsTopgg(PostBotCommandsError<crate::TwilightGetCommandsError>),
      #[cfg(feature = "bot-autoposter")]
      BotAutoposterThread(crate::Error),
    }

    struct TwilightTestContext {
      bot: TwilightHttpClient,
      running: AtomicBool,
      result_sender: mpsc::Sender<Result<(), TwilightTestError>>,
      #[cfg(feature = "bot-autoposter")]
      autoposter_receiver: Mutex<mpsc::UnboundedReceiver<crate::Result<usize>>>,
    }

    async fn test_twilight<'a>(client: &'a Client, context: &'a TwilightTestContext) -> Result<(), TwilightTestError> {
      let application_id = context.bot.current_user_application().await.map_err(TwilightTestError::PostBotCommandsTwilightApplicationIdHttp)?.model().await.map_err(TwilightTestError::PostBotCommandsTwilightApplicationIdDeserialization)?.id;
      let interaction = context.bot.interaction(application_id);

      interaction.set_global_commands(&[TwilightCommand {
        application_id: None,
        default_member_permissions: None,
        dm_permission: None,
        description: String::from("command description"),
        description_localizations: None,
        guild_id: None,
        id: None,
        kind: TwilightCommandType::ChatInput,
        name: String::from("test"),
        name_localizations: None,
        nsfw: Some(false),
        options: vec![],
        version: TwilightId::new(1)
      }]).await.map_err(TwilightTestError::PostBotCommandsTwilightSetBotCommands)?;

      client.post_bot_commands(interaction.global_commands()).await.map_err(TwilightTestError::PostBotCommandsTopgg)?;

      cfg_if::cfg_if! {
        if #[cfg(feature = "bot-autoposter")] {
          let mut bot_autopost_counter = 0;

          while let Some(posted) = context.autoposter_receiver.lock().await.recv().await {
            if let Err(err) = posted {
              return Err(TwilightTestError::BotAutoposterThread(err));
            }

            bot_autopost_counter += 1;

            if bot_autopost_counter == 3 {
              break;
            }
          }
        }
      }

      Ok(())
    }
  }
}

#[tokio::test]
async fn api() {
  let client = Client::new(env!("TOPGG_TOKEN").to_string());

  #[cfg(any(
    feature = "serenity",
    feature = "serenity-cached",
    feature = "twilight",
    feature = "twilight-cached"
  ))]
  let client = Arc::new(client);

  delayed! {
    let bot = client.get_bot(264811613708746752).await.unwrap();

    assert_eq!(bot.name, "Luca");
    assert_eq!(bot.id, 264811613708746752);
  }

  delayed! {
    let _bots = client
      .get_bots()
      .limit(250)
      .skip(50)
      .sort_by_monthly_votes()
      .await
      .unwrap();
  }

  delayed! {
    client
    .post_bot_server_count(2)
    .await
    .unwrap();
  }

  delayed! {
    assert_eq!(client.get_bot_server_count().await.unwrap().unwrap(), 2);
  }

  delayed! {
    let _voters = client.get_voters(1).await.unwrap();
  }

  delayed! {
    let _vote = client.get_vote(UserSource::Discord(661200758510977084)).await.unwrap();
  }

  delayed! {
    let _vote = client.get_vote(UserSource::Topgg(8226924471638491136)).await.unwrap();
  }

  delayed! {
    let _is_weekend = client.is_weekend().await.unwrap();
  }

  #[cfg(any(feature = "serenity", feature = "serenity-cached"))]
  delayed! {
    let bot_token = env!("BOT_TOKEN").to_string();
    let (test_result_sender, mut test_result_receiver) = mpsc::channel(1);

    cfg_if::cfg_if! {
      if #[cfg(feature = "bot-autoposter")] {
        let bot_autoposter = BotAutoposter::serenity(client.as_ref(), Duration::from_secs(2));
        let bot_autoposter_handler = bot_autoposter.handler();
      }
    }

    let bot = SerenityClient::builder(&bot_token, SerenityGatewayIntents::GUILD_MESSAGES | SerenityGatewayIntents::GUILDS)
      .event_handler(SerenityTestEventHandler {
        client: Arc::clone(&client),
        result_sender: test_result_sender,
        #[cfg(feature = "bot-autoposter")]
        bot_autoposter: Mutex::const_new(bot_autoposter),
      });

    #[cfg(feature = "bot-autoposter")]
    let bot = bot.event_handler_arc(bot_autoposter_handler);

    let mut bot = bot.await.unwrap();
    let shard_manager = Arc::clone(&bot.shard_manager);

    let test_serenity_thread = tokio::spawn(async move {
      let test_result = test_result_receiver.recv().await;

      shard_manager.shutdown_all().await;

      test_result
    });

    bot.start().await.unwrap();
    test_serenity_thread.await.unwrap().unwrap().unwrap();
  }

  #[cfg(any(feature = "twilight", feature = "twilight-cached"))]
  delayed! {
    let bot_token = env!("BOT_TOKEN").to_string();
    let (test_result_sender, mut test_result_receiver) = mpsc::channel(1);

    #[cfg(feature = "bot-autoposter")]
    let mut bot_autoposter = BotAutoposter::twilight(client.as_ref(), Duration::from_secs(2));

    let context = Arc::new(TwilightTestContext {
      bot: TwilightHttpClient::new(bot_token.clone()),
      running: AtomicBool::new(true),
      result_sender: test_result_sender,
      #[cfg(feature = "bot-autoposter")]
      autoposter_receiver: Mutex::const_new(bot_autoposter.receiver()),
    });

    let mut shard = TwilightShard::new(
      TwilightShardId::ONE,
      bot_token,
      TwilightGatewayIntents::GUILD_MESSAGES | TwilightGatewayIntents::GUILDS,
    );

    while context.running.load(atomic::Ordering::Relaxed) {
      let event = match shard.next_event().await {
        Ok(event) => event,
        Err(source) => {
          if source.is_fatal() {
            break;
          }

          continue;
        }
      };

      #[cfg(feature = "bot-autoposter")]
      bot_autoposter.handle(&event).await;

      if matches!(event, TwilightGatewayEvent::Ready(_)) {
        let thread_client = Arc::clone(&client);
        let thread_context = Arc::clone(&context);

        TWILIGHT_TEST_EVENT_HANDLER_READY_ONCE.get_or_init(|| async move {
          tokio::spawn(async move {
            let test_result = test_twilight(&thread_client, &thread_context).await;

            thread_context.running.store(false, atomic::Ordering::Relaxed);
            thread_context.result_sender.send(test_result).await.unwrap();
          });
        }).await;
      }
    }

    timeout(Duration::from_secs(10), test_result_receiver.recv()).await.unwrap().unwrap().unwrap();
  }
}
