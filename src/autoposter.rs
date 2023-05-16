use crate::{client::InnerClient, NewStats};
use std::sync::Arc;
use tokio::{
  sync::Mutex,
  task::{spawn, JoinHandle},
  time::{sleep, Duration},
};

/// A struct that lets you automate the process of posting bot statistics to the [Top.gg](https://top.gg) API in intervals.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use topgg::{Autoposter, Client, NewStats};
///
/// #[tokio::main]
/// async fn main() {
///   let token = env!("TOPGG_TOKEN").to_owned();
///   let client = Client::new(token);
///
///   // make sure to make this autoposter instance live
///   // throughout most of the bot's lifetime to keep running!
///   let autoposter = client.new_autoposter(1800);
///
///   // ... then in some on ready/new guild event ...
///   let server_count = 12345;
///   let stats = NewStats::count_based(server_count, None);
///   autoposter.feed(stats).await;
/// }
/// ```
#[must_use]
pub struct Autoposter {
  thread: JoinHandle<()>,
  data: Arc<Mutex<Option<NewStats>>>,
}

impl Autoposter {
  pub(crate) fn new(client: Arc<InnerClient>, interval: u64) -> Self {
    let current_thread_data = Arc::new(Mutex::new(None));
    let thread_data = Arc::clone(&current_thread_data);

    Self {
      thread: spawn(async move {
        loop {
          sleep(Duration::from_secs(interval)).await;

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
  /// use topgg::{Autoposter, Client, NewStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let token = env!("TOPGG_TOKEN").to_owned();
  ///   let client = Client::new(token);
  ///
  ///   // make sure to make this autoposter instance live
  ///   // throughout most of the bot's lifetime to keep running!
  ///   let autoposter = client.new_autoposter(1800);
  ///
  ///   // ... then in some on ready/new guild event ...
  ///   let server_count = 12345;
  ///   let stats = NewStats::count_based(server_count, None);
  ///   autoposter.feed(stats).await;
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
