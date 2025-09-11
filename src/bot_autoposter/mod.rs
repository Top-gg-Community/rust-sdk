use crate::Result;
use std::{ops::Deref, sync::Arc, time::Duration};
use tokio::{
  sync::mpsc,
  task::{spawn, JoinHandle},
  time::sleep,
};

mod client;

pub use client::AsClient;
pub(crate) use client::AsClientSealed;

cfg_if::cfg_if! {
  if #[cfg(feature = "serenity")] {
    mod serenity_impl;

    #[cfg_attr(docsrs, doc(cfg(feature = "serenity")))]
    pub use serenity_impl::Serenity;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "twilight")] {
    mod twilight_impl;

    #[cfg_attr(docsrs, doc(cfg(feature = "twilight")))]
    pub use twilight_impl::Twilight;
  }
}

/// Handle events from third-party Discord bot libraries.
///
/// Structs that implement this ideally should own a `RwLock<usize>` instance and update it accordingly whenever Discord sends them new data regarding their server count.
#[async_trait::async_trait]
pub trait BotAutoposterHandler: Send + Sync + 'static {
  /// The bot's latest server count.
  async fn server_count(&self) -> usize;
}

/// Automatically update the server count in your Discord bot's Top.gg page every few minutes.
///
/// **NOTE**: This struct owns the Discord bot autoposter thread which means that it will stop once it gets dropped.
///
/// # Examples
///
/// Serenity:
///
/// ```rust,no_run
/// use std::time::Duration;
/// use serenity::{client::{Client, Context, EventHandler}, model::gateway::{GatewayIntents, Ready}};
/// use topgg::BotAutoposter;
///
/// struct BotAutoposterHandler;
///
/// #[serenity::async_trait]
/// impl EventHandler for BotAutoposterHandler {
///   async fn ready(&self, _: Context, ready: Ready) {
///     println!("{} is now ready!", ready.user.name);
///   }
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
///
///   // Posts once every 30 minutes
///   let mut bot_autoposter = BotAutoposter::serenity(&client, Duration::from_secs(1800));
///   
///   let bot_token = env!("BOT_TOKEN").to_string();
///   let intents = GatewayIntents::GUILDS;
///
///   let mut bot = Client::builder(&bot_token, intents)
///     .event_handler(BotAutoposterHandler)
///     .event_handler_arc(bot_autoposter.handler())
///     .await
///     .unwrap();
///
///   let mut receiver = bot_autoposter.receiver();
///
///   tokio::spawn(async move {
///     while let Some(result) = receiver.recv().await {
///       println!("Just posted: {result:?}");
///     }
///   });
///   
///   if let Err(why) = bot.start().await {
///     println!("Client error: {why:?}");
///   }
/// }
/// ```
///
/// Twilight:
///
/// ```rust,no_run
/// use std::time::Duration;
/// use topgg::{BotAutoposter, Client};
/// use twilight_gateway::{Event, Intents, Shard, ShardId};
///
/// #[tokio::main]
/// async fn main() {
///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
///   let bot_autoposter = BotAutoposter::twilight(&client, Duration::from_secs(1800));
///
///   let mut shard = Shard::new(
///     ShardId::ONE,
///     env!("BOT_TOKEN").to_string(),
///     Intents::GUILD_MESSAGES | Intents::GUILDS,
///   );
///
///   loop {
///     let event = match shard.next_event().await {
///       Ok(event) => event,
///       Err(source) => {
///         if source.is_fatal() {
///           break;
///         }
///
///         continue;
///       }
///     };
///     
///     bot_autoposter.handle(&event).await;
///     
///     match event {
///       Event::Ready(_) => {
///         println!("Bot is now ready!");
///       },
///
///       _ => {}
///     }
///   }
/// }
/// ```
#[must_use]
pub struct BotAutoposter<H> {
  handler: Arc<H>,
  thread: JoinHandle<()>,
  receiver: Option<mpsc::UnboundedReceiver<Result<usize>>>,
}

impl<H> BotAutoposter<H>
where
  H: BotAutoposterHandler,
{
  /// Creates and starts a Discord bot autoposter thread.
  #[allow(unused_mut)]
  pub fn new<C>(client: &C, handler: H, mut interval: Duration) -> Self
  where
    C: AsClient,
  {
    #[cfg(not(test))]
    if interval.as_secs() < 900 {
      interval = Duration::from_secs(900);
    }

    let client = client.as_client();
    let handler = Arc::new(handler);
    let local_handler = Arc::clone(&handler);
    let (sender, receiver) = mpsc::unbounded_channel();

    Self {
      handler: local_handler,
      thread: spawn(async move {
        loop {
          cfg_if::cfg_if! {
            if #[cfg(test)] {
              let server_count = 3;
            } else {
              let server_count = handler.server_count().await;
            }
          }

          if sender
            .send(
              client
                .post_bot_server_count(server_count)
                .await
                .map(|()| server_count),
            )
            .is_err()
          {
            break;
          }

          sleep(interval).await;
        }
      }),
      receiver: Some(receiver),
    }
  }

  /// This Discord bot autoposter's handler.
  #[inline(always)]
  pub fn handler(&self) -> Arc<H> {
    Arc::clone(&self.handler)
  }

  /// Returns a future that resolves whenever an attempt to update the server count in your bot's Top.gg page has been made. The `usize` in this case is the server count that was just posted.
  ///
  /// **NOTE**: If you want to use the receiver directly, call [`receiver`][BotAutoposter::receiver].
  ///
  /// # Panics
  ///
  /// Panics if this method gets called again after [`receiver`][BotAutoposter::receiver] is called.
  #[inline(always)]
  pub async fn recv(&mut self) -> Option<Result<usize>> {
    self.receiver.as_mut().expect("The receiver is already taken from the receiver() method. please call recv() directly from the receiver.").recv().await
  }

  /// Takes the receiver responsible for [`recv`][BotAutoposter::recv].
  ///
  /// # Panics
  ///
  /// Panics if this method gets called for the second time.
  #[inline(always)]
  pub fn receiver(&mut self) -> mpsc::UnboundedReceiver<Result<usize>> {
    self
      .receiver
      .take()
      .expect("receiver() can only be called once.")
  }
}

impl<H> Deref for BotAutoposter<H> {
  type Target = H;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.handler
  }
}

#[cfg(feature = "serenity")]
#[cfg_attr(docsrs, doc(cfg(feature = "serenity")))]
impl BotAutoposter<Serenity> {
  /// Creates and starts a serenity-based Discord bot autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use std::time::Duration;
  /// use serenity::{client::{Client, Context, EventHandler}, model::gateway::{GatewayIntents, Ready}};
  /// use topgg::BotAutoposter;
  ///
  /// struct BotAutoposterHandler;
  ///
  /// #[serenity::async_trait]
  /// impl EventHandler for BotAutoposterHandler {
  ///   async fn ready(&self, _: Context, ready: Ready) {
  ///     println!("{} is now ready!", ready.user.name);
  ///   }
  /// }
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  ///   // Posts once every 30 minutes
  ///   let mut bot_autoposter = BotAutoposter::serenity(&client, Duration::from_secs(1800));
  ///   
  ///   let bot_token = env!("BOT_TOKEN").to_string();
  ///   let intents = GatewayIntents::GUILDS;
  ///
  ///   let mut bot = Client::builder(&bot_token, intents)
  ///     .event_handler(BotAutoposterHandler)
  ///     .event_handler_arc(bot_autoposter.handler())
  ///     .await
  ///     .unwrap();
  ///
  ///   let mut receiver = bot_autoposter.receiver();
  ///
  ///   tokio::spawn(async move {
  ///     while let Some(result) = receiver.recv().await {
  ///       println!("Just posted: {result:?}");
  ///     }
  ///   });
  ///   
  ///   if let Err(why) = bot.start().await {
  ///     println!("Client error: {why:?}");
  ///   }
  /// }
  /// ```
  #[inline(always)]
  pub fn serenity<C>(client: &C, interval: Duration) -> Self
  where
    C: AsClient,
  {
    Self::new(client, Serenity::new(), interval)
  }
}

#[cfg(feature = "twilight")]
#[cfg_attr(docsrs, doc(cfg(feature = "twilight")))]
impl BotAutoposter<Twilight> {
  /// Creates and starts a twilight-based Discord bot autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use std::time::Duration;
  /// use topgg::{BotAutoposter, Client};
  /// use twilight_gateway::{Event, Intents, Shard, ShardId};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   let bot_autoposter = BotAutoposter::twilight(&client, Duration::from_secs(1800));
  ///
  ///   let mut shard = Shard::new(
  ///     ShardId::ONE,
  ///     env!("BOT_TOKEN").to_string(),
  ///     Intents::GUILD_MESSAGES | Intents::GUILDS,
  ///   );
  ///
  ///   loop {
  ///     let event = match shard.next_event().await {
  ///       Ok(event) => event,
  ///       Err(source) => {
  ///         if source.is_fatal() {
  ///           break;
  ///         }
  ///
  ///         continue;
  ///       }
  ///     };
  ///     
  ///     bot_autoposter.handle(&event).await;
  ///     
  ///     match event {
  ///       Event::Ready(_) => {
  ///         println!("Bot is now ready!");
  ///       },
  ///
  ///       _ => {}
  ///     }
  ///   }
  /// }
  /// ```
  #[inline(always)]
  pub fn twilight<C>(client: &C, interval: Duration) -> Self
  where
    C: AsClient,
  {
    Self::new(client, Twilight::new(), interval)
  }
}

impl<H> Drop for BotAutoposter<H> {
  #[inline(always)]
  fn drop(&mut self) {
    self.thread.abort();
  }
}
