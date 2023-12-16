use crate::{client::InnerClient, Stats};
use core::{ops::Deref, time::Duration};
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

/// A fully [`Clone`]able and thread-safe struct that lets you remotely feed bot statistics to the [`Autoposter`].
pub struct AutoposterHandle {
  data: Arc<Mutex<PendingData>>,
}

impl AutoposterHandle {
  /// Feeds new bot stats to this autoposter handle. The [autoposter itself][Autoposter] will automatically post it to [Top.gg](https://top.gg) servers once appropiate.
  ///
  /// # Examples
  ///
  /// Direct usage with an [`Autoposter`]:
  ///
  /// ```rust,no_run
  /// use core::time::Duration;
  /// use topgg::{Client, Stats};
  ///
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// // creates an autoposter that posts data to Top.gg every 1800 seconds (30 minutes).
  /// // the autopost thread will stop once it's dropped.
  /// let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  /// // ... then in some on ready/new guild event ...
  /// let server_count = 12345;
  /// let stats = Stats::from(server_count);
  /// autoposter.feed(stats).await;
  /// ```
  ///
  /// Remote usage with an [`AutoposterHandle`]:
  ///
  /// ```rust,no_run
  /// use core::time::Duration;
  /// use topgg::{Client, Stats};
  ///
  /// let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  /// // creates an autoposter that posts data to Top.gg every 1800 seconds (30 minutes).
  /// // the autopost thread will stop once it's dropped.
  /// let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  /// let server_count = 12345;
  /// autoposter
  ///   .feed(Stats::from(server_count))
  ///   .await;
  ///
  /// // this handle can be cloned and tossed around threads!
  /// let new_handle = autoposter.handle();
  ///
  /// // do the same thing...
  /// new_handle
  ///   .feed(Stats::from(server_count))
  ///   .await;
  ///
  /// let another_handle = new_handle.clone();
  ///
  /// // do the same thing...
  /// another_handle
  ///   .feed(Stats::from(server_count))
  ///   .await;
  /// ```
  pub async fn feed(&self, new_stats: Stats) {
    let mut lock = self.data.lock().await;

    lock.ready = true; // flag the PendingData object as containing new data.
    lock.stats = new_stats;
  }
}

/// Creates another handle that points to the same reference handle. Somewhat similar to an [`Arc::clone`].
impl Clone for AutoposterHandle {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      data: Arc::clone(&self.data),
    }
  }
}

/// A struct that lets you automate the process of posting bot statistics to [Top.gg](https://top.gg) in intervals.
///
/// **NOTE:** This struct owns the thread handle that executes the automatic posting. The autoposter thread will stop once this struct is dropped.
#[must_use]
pub struct Autoposter {
  thread: JoinHandle<()>,
  handle: AutoposterHandle,
}

impl Autoposter {
  #[allow(invalid_value, clippy::uninit_assumed_init)]
  pub(crate) fn new(client: Arc<InnerClient>, interval: Duration) -> Self {
    let handle = AutoposterHandle {
      data: Arc::new(Mutex::new(PendingData {
        ready: false,
        stats: Stats::count_based(0, None),
      })),
    };

    let thread_data = Arc::clone(&handle.data);

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
      handle,
    }
  }

  /// Creates an [`AutoposterHandle`] that lets you remotely feed bot statistics to this [`Autoposter`]. This struct is fully [`Clone`]able and thread-safe.
  #[inline(always)]
  pub fn handle(&self) -> AutoposterHandle {
    self.handle.clone()
  }
}

impl Deref for Autoposter {
  type Target = AutoposterHandle;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.handle
  }
}

impl Drop for Autoposter {
  #[inline(always)]
  fn drop(&mut self) {
    self.thread.abort();
  }
}
