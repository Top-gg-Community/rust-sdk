use crate::{client::InnerClient, Stats};
use core::{ops::Deref, time::Duration};
use std::sync::Arc;
use tokio::{
  sync::{Mutex, Semaphore},
  task::{spawn, JoinHandle},
  time::sleep,
};

struct Data {
  stats: Mutex<Stats>,
  sem: Semaphore,
}

/// A fully [`Clone`]able and thread-safe struct that lets you remotely feed bot statistics to the [`Autoposter`].
pub struct AutoposterHandle {
  data: Arc<Data>,
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
  /// autoposter.feed(Stats::from(server_count)).await;
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
    {
      let mut lock = self.data.stats.lock().await;
      *lock = new_stats;
    };

    if self.data.sem.available_permits() == 0 {
      self.data.sem.add_permits(1);
    }
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
  pub(crate) fn new(client: Arc<InnerClient>, interval: Duration) -> Self {
    let data = Arc::new(Data {
      stats: Mutex::const_new(Stats::from(0)),
      sem: Semaphore::const_new(0),
    });

    let handle = AutoposterHandle { data: data.clone() };

    Self {
      thread: spawn(async move {
        loop {
          {
            let _ = data.sem.acquire().await.unwrap();
            let lock = data.stats.lock().await;
            let _ = client.post_stats(&lock).await;
          };

          sleep(interval).await;
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
