use crate::Result;
use core::{ops::Deref, time::Duration};
use std::sync::Arc;
use tokio::{
  sync::{mpsc, RwLock},
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

/// Handle events from third-party bot libraries.
///
/// Structs that implement this ideally should own a `RwLock<usize>` instance and update it accordingly whenever Discord sends them new data regarding their server count.
pub trait Handler: Send + Sync + 'static {
  /// Borrows the instance to the [`Autoposter`].
  fn server_count(&self) -> &RwLock<usize>;
}

/// Automate the process of posting your bot's server count to the API.
///
/// **NOTE**: This struct owns the thread that does the autoposting. It will stop once it gets dropped.
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
  /// Creates an autoposter instance and immediately starts up the thread.
  ///
  /// - `client` can either be a reference to an existing [`Client`][crate::Client] or an API token ([`&str`][std::str]).
  /// - `handler` is any struct that gives out server count information.
  /// - `interval` is the interval between posting. Defaults to 15 minutes.
  pub fn new<C>(client: &C, handler: H, mut interval: Duration) -> Self
  where
    C: AsClient,
  {
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
            let server_count = handler.server_count().read().await;

            if sender
              .send(client.post_server_count(*server_count).await)
              .is_err()
            {
              break;
            }
          };

          sleep(interval).await;
        }
      }),
      receiver: Some(receiver),
    }
  }

  /// Retrieves this autoposter's handler.
  #[inline(always)]
  pub fn handler(&self) -> Arc<H> {
    Arc::clone(&self.handler)
  }

  /// Returns a future that resolves every time the autoposter posts your bot's server count.
  ///
  /// If you want to use the receiver directly, call [`receiver`][Autoposter::receiver].
  ///
  /// # Panics
  ///
  /// Subsequent calls to this method.
  #[inline(always)]
  pub async fn recv(&mut self) -> Option<Result<()>> {
    self.receiver.as_mut().expect("The receiver is already taken from the receiver() method. please call recv() directly from the receiver.").recv().await
  }

  /// Takes the receiver responsible for [`recv`][Autoposter::recv].
  ///
  /// # Panics
  ///
  /// Subsequent calls to this method.
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
  /// Creates a serenity-based autoposter instance and immediately starts up the thread.
  ///
  /// - `client` can either be a reference to an existing [`Client`][crate::Client] or an API token ([`&str`][std::str]).
  /// - `interval` is the interval between posting. Defaults to 15 minutes.
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
  /// Creates a twilight-based autoposter instance and immediately starts up the thread.
  ///
  /// - `client` can either be a reference to an existing [`Client`][crate::Client] or an API token ([`&str`][std::str]).
  /// - `interval` is the interval between posting. Defaults to 15 minutes.
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
