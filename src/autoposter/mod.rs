use crate::{Result, Stats};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::Duration,
};
use tokio::{
  sync::{mpsc, RwLock, RwLockWriteGuard},
  task::{spawn, JoinHandle},
  time::sleep,
};

mod client;

pub use client::AsClient;
pub(crate) use client::AsClientSealed;

cfg_if::cfg_if! {
  if #[cfg(any(feature = "serenity", feature = "serenity-cached"))] {
    mod serenity_impl;

    #[cfg_attr(docsrs, doc(cfg(any(feature = "serenity", feature = "serenity-cached"))))]
    pub use serenity_impl::Serenity;
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "twilight", feature = "twilight-cached"))] {
    mod twilight_impl;

    #[cfg_attr(docsrs, doc(cfg(any(feature = "twilight", feature = "twilight-cached"))))]
    pub use twilight_impl::Twilight;
  }
}

/// A thread-safe form of the [`Stats`] struct to be used in autoposter [`Handler`]s.
pub struct SharedStats {
  stats: RwLock<Stats>,
}

/// A guard wrapping over tokio's [`RwLockWriteGuard`] that lets you freely feed new [`Stats`] data before being sent to the [`Autoposter`].
pub struct SharedStatsGuard<'a> {
  guard: RwLockWriteGuard<'a, Stats>,
}

impl SharedStatsGuard<'_> {
  /// Directly replaces the current [`Stats`] inside with another.
  #[inline(always)]
  pub fn replace(&mut self, other: Stats) {
    *self.guard = other;
  }

  /// Sets the current [`Stats`] server count.
  #[inline(always)]
  #[cfg_attr(any(test, feature = "_internal-doctest"), allow(unused_variables))]
  pub fn set_server_count(&mut self, server_count: usize) {
    cfg_if::cfg_if! {
      if #[cfg(any(test, feature = "_internal-doctest"))] {
        self.guard.server_count = Some(2);
      } else {
        self.guard.server_count = Some(server_count);
      }
    }
  }

  /// Sets the current [`Stats`] shard count.
  #[inline(always)]
  pub fn set_shard_count(&mut self, shard_count: usize) {
    self.guard.shard_count = Some(shard_count);
  }
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

impl SharedStats {
  /// Creates a new [`SharedStats`] struct. Before any modifications, the [`Stats`] struct inside defaults to zero server count.
  #[inline(always)]
  pub fn new() -> Self {
    Self {
      stats: RwLock::new(Stats::from(0)),
    }
  }

  /// Locks this [`SharedStats`] with exclusive write access, causing the current task to yield until the lock has been acquired. This is akin to [`RwLock::write`].
  #[inline(always)]
  pub async fn write(&self) -> SharedStatsGuard<'_> {
    SharedStatsGuard {
      guard: self.stats.write().await,
    }
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
/// The struct implementing this trait should own an [`SharedStats`] struct and update it accordingly whenever Discord updates them with new data regarding server/shard count.
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
  #[cfg_attr(any(test, feature = "_internal-doctest"), allow(unused_mut))]
  pub fn new<C>(client: &C, handler: H, mut interval: Duration) -> Self
  where
    C: AsClient,
  {
    #[cfg(not(any(test, feature = "_internal-doctest")))]
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

#[cfg(any(feature = "serenity", feature = "serenity-cached"))]
#[cfg_attr(
  docsrs,
  doc(cfg(any(feature = "serenity", feature = "serenity-cached")))
)]
impl Autoposter<Serenity> {
  /// Creates and starts a serenity-based autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::time::Duration;
  /// use serenity::{client::{Client, Context, EventHandler}, model::gateway::{GatewayIntents, Ready}};
  /// use topgg::Autoposter;
  /// #
  /// # use std::sync::Arc;
  /// # use tokio::sync::{mpsc, Mutex, Notify};
  /// #
  /// # struct SerenityTestContext {
  /// #   immature_thread_closure: Notify,
  /// #   autoposter_receiver: Mutex<mpsc::UnboundedReceiver<topgg::Result<()>>>,
  /// # }
  /// #
  /// # impl SerenityTestContext {
  /// #   async fn autoposter_recv(&self) -> Option<topgg::Result<()>> {
  /// #     let mut guard = self.autoposter_receiver.lock().await;
  /// #     
  /// #     guard.recv().await
  /// #   }
  /// # }
  ///
  /// struct AutoposterHandler;
  ///
  /// #[async_trait::async_trait]
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
  /// # /*
  ///   let mut autoposter = Autoposter::serenity(&client, Duration::from_secs(1800));
  /// # */
  /// # let mut autoposter = Autoposter::serenity(&client, Duration::from_secs(2));
  /// #
  /// # let local_test_context = Arc::new(SerenityTestContext {
  /// #   immature_thread_closure: Notify::const_new(),
  /// #   autoposter_receiver: Mutex::const_new(autoposter.receiver()),
  /// # });
  /// # let thread_test_context = Arc::clone(&local_test_context);
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
  /// # let shard_manager = Arc::clone(&bot.shard_manager);
  /// # /*
  ///   let mut receiver = autoposter.receiver();
  ///
  ///   tokio::spawn(async move {
  /// # */
  /// # let test_thread = tokio::spawn(async move {
  /// #   let mut autopost_counter = 0;
  /// #
  /// #   /*
  ///     while let Some(result) = receiver.recv().await {
  /// #   */
  /// #   loop {
  /// #     tokio::select! {
  /// #       _ = thread_test_context.immature_thread_closure.notified() => {
  /// #         return Ok(());
  /// #       }
  /// #     
  /// #       Some(result) = thread_test_context.autoposter_recv() => {
  ///       println!("Just posted: {result:?}");
  /// #         autopost_counter += 1;
  /// #       
  /// #         if result.is_err() || autopost_counter == 3 {
  /// #           shard_manager.shutdown_all().await;
  /// #           
  /// #           return result;
  /// #         }
  /// #       }
  /// #     }
  ///     }
  ///   });
  ///   
  ///   if let Err(why) = bot.start().await {
  /// #   local_test_context.immature_thread_closure.notify_one();
  /// #   
  /// #   /*
  ///     println!("Client error: {why:?}");
  /// #   */
  /// #   panic!("Client error: {why:?}");
  ///   }
  /// #
  /// # test_thread.await.unwrap().unwrap();
  /// }
  ///
  /// ```
  #[inline(always)]
  pub fn serenity<C>(client: &C, interval: Duration) -> Self
  where
    C: AsClient,
  {
    Self::new(client, Serenity::new(), interval)
  }
}

#[cfg(any(feature = "twilight", feature = "twilight-cached"))]
#[cfg_attr(
  docsrs,
  doc(cfg(any(feature = "twilight", feature = "twilight-cached")))
)]
impl Autoposter<Twilight> {
  /// Creates and starts a twilight-based autoposter thread.
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::time::Duration;
  /// use topgg::{Autoposter, Client};
  /// use twilight_gateway::{Event, Intents, Shard, ShardId};
  /// #
  /// # use std::sync::{atomic::{self, AtomicBool}, Arc};
  /// # use tokio::sync::{mpsc, Mutex, Notify, OnceCell};
  /// #
  /// # static TEST_EVENT_HANDLER_READY_ONCE: OnceCell<()> = OnceCell::const_new();
  /// #
  /// # enum TwilightTestError {
  /// #   FatalImmatureClosure,
  /// #   PostStats(topgg::Error),
  /// # }
  /// #
  /// # struct TwilightTestContext {
  /// #   running: AtomicBool,
  /// #   immature_thread_closure: Notify,
  /// #   test_result_sender: mpsc::Sender<Result<(), TwilightTestError>>,
  /// #   autoposter_receiver: Mutex<mpsc::UnboundedReceiver<topgg::Result<()>>>,
  /// # }
  /// #
  /// # impl TwilightTestContext {
  /// #   async fn autoposter_recv(&self) -> Option<topgg::Result<()>> {
  /// #     let mut guard = self.autoposter_receiver.lock().await;
  /// #     
  /// #     guard.recv().await
  /// #   }
  /// # }
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  /// # /*
  ///   let autoposter = Autoposter::twilight(&client, Duration::from_secs(1800));
  /// # */
  /// # let mut autoposter = Autoposter::twilight(&client, Duration::from_secs(2));
  /// #
  /// # let (test_result_sender, mut test_result_receiver) = mpsc::channel(1);
  /// #
  /// # let local_test_context = Arc::new(TwilightTestContext {
  /// #   running: AtomicBool::new(true),
  /// #   immature_thread_closure: Notify::const_new(),
  /// #   test_result_sender,
  /// #   autoposter_receiver: Mutex::const_new(autoposter.receiver()),
  /// # });
  ///
  ///   let mut shard = Shard::new(
  ///     ShardId::ONE,
  ///     env!("BOT_TOKEN").to_string(),
  ///     Intents::GUILD_MESSAGES | Intents::GUILDS,
  ///   );
  ///
  ///   loop {
  /// #   if !local_test_context.running.load(atomic::Ordering::Relaxed) {
  /// #     break;
  /// #   }
  /// #   
  ///     let event = match shard.next_event().await {
  ///       Ok(event) => event,
  ///       Err(source) => {
  ///         if source.is_fatal() {
  /// #         local_test_context.immature_thread_closure.notify_one();
  /// #         
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
  /// #       
  /// #       let thread_test_context = Arc::clone(&local_test_context);
  /// #       
  /// #       TEST_EVENT_HANDLER_READY_ONCE.get_or_init(|| async move {
  /// #         tokio::spawn(async move {
  /// #           let mut autopost_counter = 0;
  /// #           
  /// #           loop {
  /// #             tokio::select! {
  /// #               _ = thread_test_context.immature_thread_closure.notified() => {
  /// #                 thread_test_context.test_result_sender.send(Err(TwilightTestError::FatalImmatureClosure)).await.unwrap();
  /// #                 
  /// #                 return;
  /// #               }
  /// #               
  /// #               Some(posted) = thread_test_context.autoposter_recv() => {
  /// #                 autopost_counter += 1;
  /// #                 
  /// #                 if posted.is_err() || autopost_counter == 3 {
  /// #                   thread_test_context.test_result_sender.send(posted.map_err(TwilightTestError::PostStats)).await.unwrap();
  /// #                   thread_test_context.running.store(false, atomic::Ordering::Relaxed);
  /// #                   
  /// #                   return;
  /// #                 }
  /// #               }
  /// #             }
  /// #           }
  /// #         });
  /// #       }).await;
  ///       },
  ///
  ///       _ => {}
  ///     }
  ///   }
  /// #
  /// # test_result_receiver.recv().await.unwrap().unwrap();
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
