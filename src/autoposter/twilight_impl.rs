use crate::autoposter::{Handler, SharedStats};
use std::{collections::HashSet, ops::DerefMut};
use tokio::sync::Mutex;
use twilight_model::gateway::event::Event;

/// A built-in [`Handler`] for the [twilight](https://twilight.rs) library.
pub struct Twilight {
  cache: Mutex<HashSet<u64>>,
  stats: SharedStats,
}

impl Twilight {
  #[inline(always)]
  pub(super) fn new() -> Self {
    Self {
      cache: Mutex::const_new(HashSet::new()),
      stats: SharedStats::new(),
    }
  }

  /// Handles an entire [twilight](https://twilight.rs) [`Event`] enum.
  pub async fn handle(&self, event: &Event) {
    match event {
      Event::Ready(ready) => {
        let mut cache = self.cache.lock().await;
        let mut stats = self.stats.write().await;
        let cache_ref = cache.deref_mut();

        *cache_ref = ready.guilds.iter().map(|guild| guild.id.get()).collect();
        stats.set_server_count(cache.len());
      }

      Event::GuildCreate(guild_create) => {
        let mut cache = self.cache.lock().await;

        if cache.insert(guild_create.0.id.get()) {
          let mut stats = self.stats.write().await;

          stats.set_server_count(cache.len());
        }
      }

      Event::GuildDelete(guild_delete) => {
        let mut cache = self.cache.lock().await;

        if cache.remove(&guild_delete.id.get()) {
          let mut stats = self.stats.write().await;

          stats.set_server_count(cache.len());
        }
      }

      _ => {}
    }
  }
}

impl Handler for Twilight {
  #[inline(always)]
  fn stats(&self) -> &SharedStats {
    &self.stats
  }
}
