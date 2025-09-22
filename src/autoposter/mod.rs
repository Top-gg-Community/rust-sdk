use crate::{Result, Stats};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::Duration,
};
use tokio::{
  sync::{mpsc, RwLock, RwLockWriteGuard, Semaphore},
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

/// A thread-safe form of the [`Stats`] struct to be used in autoposter [`Handler`]s.
pub struct SharedStats {
  sem: Semaphore,
  stats: RwLock<Stats>,
}

/// A guard wrapping over tokio's [`RwLockWriteGuard`] that lets you freely feed new [`Stats`] data before being sent to the [`Autoposter`].
pub struct SharedStatsGuard<'a> {
  sem: &'a Semaphore,
  guard: RwLockWriteGuard<'a, Stats>,
}

impl SharedStatsGuard<'_> {
  /// Directly replaces the current [`Stats`] inside with the other.
  #[inline(always)]
  pub fn replace(&mut self, other: Stats) {
    *self.guard = other;
  }

  /// Sets the current [`Stats`] server count.
  #[inline(always)]
  pub fn set_server_count(&mut self, server_count: usize) {
    self.guard.server_count = Some(server_count);
  }

  /// Sets the current [`Stats`] shard count.
  #[deprecated(since = "1.5.0", note = "No longer supported by API v0.")]
  pub fn set_shard_count(&mut self, _shard_count: usize) {}
}

impl Deref for SharedStatsGuard<'_> {
  type Target = Stats;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.guard
  }
}

impl DerefMut for SharedStatsGuard<'_> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.guard
  }
}

impl Drop for SharedStatsGuard<'_> {
  #[inline(always)]
  fn drop(&mut self) {
    if self.sem.available_permits() < 1 {
      self.sem.add_permits(1);
    }
  }
}

impl SharedStats {
  /// Creates a new [`SharedStats`] struct. Before any modifications, the [`Stats`] struct inside defaults to zero server count.
  #[inline(always)]
  pub fn new() -> Self {
    Self {
      sem: Semaphore::const_new(0),
      stats: RwLock::new(Stats::from(0)),
    }
  }

  /// Locks this [`SharedStats`] with exclusive write access, causing the current task to yield until the lock has been acquired. This is akin to [`RwLock::write`].
  #[inline(always)]
  pub async fn write(&self) -> SharedStatsGuard<'_> {
    SharedStatsGuard {
      sem: &self.sem,
      guard: self.stats.write().await,
    }
  }

  #[inline(always)]
  async fn wait(&self) {
    self.sem.acquire().await.unwrap().forget();
  }
}

impl Default for SharedStats {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

/// A trait for handling events from third-party Discord Bot libraries.
///
/// The struct implementing this trait should own an [`SharedStats`] struct and update it accordingly whenever Discord updates them with new data regarding guild count.
pub trait Handler: Send + Sync + 'static {
  /// The method that borrows [`SharedStats`] to the [`Autoposter`].
  fn stats(&self) -> &SharedStats;
}

/// Automatically update the stats in your Discord bot's Top.gg page every few minutes.
///
/// **NOTE**: This struct owns the autoposter thread which means that it will stop once it gets dropped.
#[must_use]
pub struct Autoposter<H> {
  handler: Arc<H>,
  thread: JoinHandle<()>,
  receiver: Option<mpsc::UnboundedReceiver<Result<()>>>,
}

impl<H> Autoposter<H>
where
  H: Handler,
{
  /// Creates and starts an autoposter thread.
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
    let (sender, receiver) = mpsc::unbounded_channel();

    Self {
      handler: Arc::clone(&handler),
      thread: spawn(async move {
        loop {
          handler.stats().wait().await;

          {
            let stats = handler.stats().stats.read().await;

            if sender.send(client.post_stats(&stats).await).is_err() {
              break;
            }
          };

          sleep(interval).await;
        }
      }),
      receiver: Some(receiver),
    }
  }

  /// This autoposter's handler.
  #[inline(always)]
  pub fn handler(&self) -> Arc<H> {
    Arc::clone(&self.handler)
  }

  /// Returns a future that resolves whenever an attempt to update the stats in your bot's Top.gg page has been made.
  ///
  /// **NOTE**: If you want to use the receiver directly, call [`receiver`][Autoposter::receiver].
  ///
  /// # Panics
  ///
  /// Panics if this method gets called again after [`receiver`][Autoposter::receiver] is called.
  #[inline(always)]
  pub async fn recv(&mut self) -> Option<Result<()>> {
    self.receiver.as_mut().expect("The receiver is already taken from the receiver() method. please call recv() directly from the receiver.").recv().await
  }

  /// Takes the receiver responsible for [`recv`][Autoposter::recv].
  ///
  /// # Panics
  ///
  /// Panics if this method gets called for the second time.
  #[inline(always)]
  pub fn receiver(&mut self) -> mpsc::UnboundedReceiver<Result<()>> {
    self
      .receiver
      .take()
      .expect("receiver() can only be called once.")
  }
}

impl<H> Deref for Autoposter<H> {
  type Target = H;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.handler
  }
}

#[cfg(feature = "serenity")]
#[cfg_attr(docsrs, doc(cfg(feature = "serenity")))]
impl Autoposter<Serenity> {
  /// Creates and starts a serenity-based autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use std::time::Duration;
  /// use serenity::{client::{Client, Context, EventHandler}, model::gateway::{GatewayIntents, Ready}};
  /// use topgg::Autoposter;
  ///
  /// struct AutoposterHandler;
  ///
  /// #[serenity::async_trait]
  /// impl EventHandler for AutoposterHandler {
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
  ///   let mut autoposter = Autoposter::serenity(&client, Duration::from_secs(1800));
  ///   
  ///   let bot_token = env!("BOT_TOKEN").to_string();
  ///   let intents = GatewayIntents::GUILDS;
  ///
  ///   let mut bot = Client::builder(&bot_token, intents)
  ///     .event_handler(AutoposterHandler)
  ///     .event_handler_arc(autoposter.handler())
  ///     .await
  ///     .unwrap();
  ///
  ///   let mut receiver = autoposter.receiver();
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
impl Autoposter<Twilight> {
  /// Creates and starts a twilight-based autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use std::time::Duration;
  /// use topgg::{Autoposter, Client};
  /// use twilight_gateway::{Event, Intents, Shard, ShardId};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///   let autoposter = Autoposter::twilight(&client, Duration::from_secs(1800));
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
  ///     autoposter.handle(&event).await;
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

impl<H> Drop for Autoposter<H> {
  #[inline(always)]
  fn drop(&mut self) {
    self.thread.abort();
  }
}
