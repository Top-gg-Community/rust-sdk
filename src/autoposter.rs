use crate::{client::InnerClient, Stats};
use core::{mem::MaybeUninit, time::Duration};
use std::sync::Arc;
use tokio::{
  sync::Mutex,
  task::{spawn, JoinHandle},
  time::sleep,
};

struct PendingData {
  ready: bool,
  stats: Stats,
}

/// A struct that lets you automate the process of posting bot statistics to the [Top.gg API](https://docs.top.gg) in intervals.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use core::time::Duration;
/// use topgg::{Client, Stats};
///
/// #[tokio::main]
/// async fn main() {
///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
///
///   // creates an autoposter that posts data to Top.gg every 1800 seconds (30 minutes).
///   // the autopost thread will stop once it's dropped.
///   let autoposter = client.new_autoposter(Duration::from_secs(1800));
///
///   // ... then in some on ready/new guild event ...
///   let server_count = 12345;
///   autoposter
///     .feed(Stats::count_based(server_count, None))
///     .await;
/// }
/// ```
#[must_use]
pub struct Autoposter {
  thread: JoinHandle<()>,
  data: Arc<Mutex<PendingData>>,
}

impl Autoposter {
  #[allow(invalid_value, clippy::uninit_assumed_init)]
  pub(crate) fn new(client: Arc<InnerClient>, interval: Duration) -> Self {
    // SAFETY: post_stats will be called ONLY when the ready flag is set to true.
    let current_thread_data = Arc::new(Mutex::new(PendingData {
      ready: false,
      stats: unsafe { MaybeUninit::uninit().assume_init() },
    }));

    let thread_data = Arc::clone(&current_thread_data);

    Self {
      thread: spawn(async move {
        loop {
          sleep(interval).await;

          let mut lock = thread_data.lock().await;

          if lock.ready {
            let _ = client.post_stats(&lock.stats).await;
            lock.ready = false; // flag the PendingData object as out-of-date.
          }
        }
      }),
      data: current_thread_data,
    }
  }

  /// Feeds new bot stats to the autoposter. The autoposter will automatically post it to the [Top.gg](https://top.gg) servers in intervals.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use core::time::Duration;
  /// use topgg::{Client, Stats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  ///   // creates an autoposter that posts data to Top.gg every 1800 seconds (30 minutes).
  ///   // the autopost thread will stop once it's dropped.
  ///   let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  ///   // ... then in some on ready/new guild event ...
  ///   let server_count = 12345;
  ///   autoposter
  ///     .feed(Stats::count_based(server_count, None))
  ///     .await;
  /// }
  /// ```
  pub async fn feed(&self, new_stats: Stats) {
    let mut lock = self.data.lock().await;

    lock.ready = true; // flag the PendingData object as containing new data.
    lock.stats = new_stats;
  }
}

impl Drop for Autoposter {
  #[inline(always)]
  fn drop(&mut self) {
    self.thread.abort();
  }
}
