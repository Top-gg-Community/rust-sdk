use crate::{client::InnerClient, NewBotStats};
use std::sync::Arc;
use tokio::{
  sync::Mutex,
  task::{spawn, JoinHandle},
  time::{sleep, Duration},
};

///	A struct that lets you automate the process of posting bot statistics to the [top.gg](https://top.gg) API.
pub struct Autoposter {
  thread: JoinHandle<!>,
  data: Arc<Mutex<Option<NewBotStats>>>,
}

impl Autoposter {
  #[allow(unused_must_use)]
  pub(crate) fn new(client: &Arc<InnerClient>, id: u64, delay: u64) -> Self {
    let current_thread_data = Arc::new(Mutex::new(None));
    let thread_client = Arc::clone(client);
    let thread_data = Arc::clone(&current_thread_data);

    Self {
      thread: spawn(async move {
        loop {
          sleep(Duration::from_secs(delay)).await;

          let lock = thread_data.lock().await;

          if let Some(new_data) = &*lock {
            thread_client.post_bot_stats(id, new_data).await;
          }
        }
      }),
      data: current_thread_data,
    }
  }

  /// Feeds new bot stats to the autoposter. The autoposter will automatically post it to the [top.gg](https://top.gg) servers once the delay is complete.
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use std::env;
  /// use topgg::{Autoposter, Client, NewBotStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let token = env::var("TOPGG_TOKEN").expect("missing top.gg token");
  ///   let client = Client::new(token);
  ///   let my_bot_id = 123456789u64;
  ///
  ///   // make sure to make this autoposter instance live
  ///   // throughout most of the bot's lifetime to keep running!
  ///   let autoposter = client.new_autoposter(my_bot_id, 1800);
  ///
  ///   // ... then in some on ready/new guild event ...
  ///   let server_count = 12345;
  ///   let stats = NewBotStats::count_based(server_count, None);
  ///   autoposter.feed(stats).await;
  /// }
  /// ```
  pub async fn feed(&mut self, new_stats: NewBotStats) {
    let mut lock = self.data.lock().await;

    (*lock).replace(new_stats);
  }
}

impl Drop for Autoposter {
  fn drop(&mut self) {
    self.thread.abort();
  }
}
