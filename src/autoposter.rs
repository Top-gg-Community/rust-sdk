use crate::{client::InnerClient, NewStats};
use core::time::Duration;
use std::sync::Arc;
use tokio::{
  sync::Mutex,
  task::{spawn, JoinHandle},
  time::sleep,
};

/// A struct that lets you automate the process of posting bot statistics to the [Top.gg](https://top.gg) API in intervals.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use core::time::Duration;
/// use topgg::{Autoposter, Client, NewStats};
///
/// #[tokio::main]
/// async fn main() {
///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
///
///   // creates an autoposter that posts data to Top.gg every 1800 seconds (15 minutes).
///   // the autopost thread will stop once it's dropped.
///   let autoposter = client.new_autoposter(Duration::from_secs(1800));
///
///   // ... then in some on ready/new guild event ...
///   let server_count = 12345;
///   autoposter.feed(NewStats::count_based(server_count, None)).await;
/// }
/// ```
#[must_use]
pub struct Autoposter {
  thread: JoinHandle<()>,
  data: Arc<Mutex<Option<NewStats>>>,
}

impl Autoposter {
  pub(crate) fn new(client: Arc<InnerClient>, interval: Duration) -> Self {
    let current_thread_data = Arc::new(Mutex::new(None));
    let thread_data = current_thread_data.clone();

    Self {
      thread: spawn(async move {
        loop {
          sleep(interval).await;

          let lock = thread_data.lock().await;

          if let Some(new_data) = &*lock {
            let _ = client.post_stats(new_data).await;
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
  /// use topgg::{Autoposter, Client, NewStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = Client::new(env!("TOPGG_TOKEN").to_string());
  ///
  ///   // creates an autoposter that posts data to Top.gg every 1800 seconds (15 minutes).
  ///   // the autopost thread will stop once it's dropped.
  ///   let autoposter = client.new_autoposter(Duration::from_secs(1800));
  ///
  ///   // ... then in some on ready/new guild event ...
  ///   let server_count = 12345;
  ///   autoposter.feed(NewStats::count_based(server_count, None)).await;
  /// }
  /// ```
  #[inline(always)]
  pub async fn feed(&self, new_stats: NewStats) {
    (*self.data.lock().await).replace(new_stats);
  }
}

impl Drop for Autoposter {
  #[inline(always)]
  fn drop(&mut self) {
    self.thread.abort();
  }
}
