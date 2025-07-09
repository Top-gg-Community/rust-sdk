use crate::autoposter::Handler;
use std::collections::HashSet;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use twilight_model::gateway::event::Event;

/// Autoposter handler for working with the twilight.
pub struct Twilight {
  cache: Mutex<HashSet<u64>>,
  server_count: RwLock<usize>,
}

impl Twilight {
  #[inline(always)]
  pub(super) fn new() -> Self {
    Self {
      cache: Mutex::const_new(HashSet::new()),
      server_count: RwLock::new(0),
    }
  }

  /// Handles an entire twilight [`Event`] enum.
  pub async fn handle(&self, event: &Event) {
    match event {
      Event::Ready(ready) => {
        let mut cache: tokio::sync::MutexGuard<'_, HashSet<u64>> = self.cache.lock().await;
        let mut server_count = self.server_count.write().await;
        let cache_ref = &mut *cache;

        *cache_ref = ready.guilds.iter().map(|guild| guild.id.get()).collect();
        *server_count = cache.len();
      }

      Event::GuildCreate(guild_create) => {
        let mut cache = self.cache.lock().await;

        if cache.insert(guild_create.id().get()) {
          let mut server_count = self.server_count.write().await;

          *server_count = cache.len();
        }
      }

      Event::GuildDelete(guild_delete) => {
        let mut cache = self.cache.lock().await;

        if cache.remove(&guild_delete.id.get()) {
          let mut server_count = self.server_count.write().await;

          *server_count = cache.len();
        }
      }

      _ => {}
    }
  }
}

impl<'a> Handler<'a> for Twilight {
  #[inline(always)]
  fn server_count(&'a self) -> RwLockReadGuard<'a, usize> {
    self.server_count.read()
  }
}
