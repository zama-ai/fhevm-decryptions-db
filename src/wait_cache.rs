// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use async_cell::sync::AsyncCell;
use moka::sync::Cache;

pub struct WaitCache<K, V> {
    cache: Cache<K, Arc<AsyncCell<V>>>,
}

impl<K: Eq + Send + Sync + Hash + 'static, V: Clone + Sync + Send + 'static> WaitCache<K, V> {
    /// Create a new wait cache. Key-values will remain in the cache for `time_to_live` duration.
    pub fn new(time_to_live: Duration) -> Self {
        WaitCache {
            cache: Cache::builder().time_to_live(time_to_live).build(),
        }
    }

    /// Put a key-value into the cache. Signals any readers blocked on `get_timeout()`.
    pub fn put(&self, key: K, value: V) {
        self.cache.get_with(key, AsyncCell::shared).set(value);
    }

    /// Get a key-value from the cache.
    ///
    /// If the key is not present at the time of the call,
    /// the returned future will resolve when the key becomes available.
    ///
    /// If the key doesn't become available after `timeout` duration has passed, None is returned.
    pub async fn get_timeout(&self, key: K, timeout: Duration) -> Option<V> {
        let cell = self.cache.get_with(key, AsyncCell::shared);
        rocket::tokio::time::timeout(timeout, cell.get_shared())
            .await
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use rocket::tokio;

    use super::*;

    #[tokio::test]
    async fn put_get() {
        let cache = WaitCache::<u64, u64>::new(Duration::from_secs(30));
        let key = 1;
        let value = 2;
        cache.put(key, value);
        let get_value = cache.get_timeout(key, Duration::from_secs(1)).await;
        assert_eq!(get_value, Some(value));
    }

    #[tokio::test]
    async fn get_put() {
        let cache = WaitCache::<u64, u64>::new(Duration::from_secs(30));
        let key = 1;
        let value = 2;
        let get_value = cache.get_timeout(key, Duration::from_secs(5));
        cache.put(key, value);
        assert_eq!(get_value.await, Some(value));
    }

    #[tokio::test]
    async fn two_gets_then_put() {
        let cache = WaitCache::<u64, u64>::new(Duration::from_secs(30));
        let key = 1;
        let value = 2;
        let get_value1 = cache.get_timeout(key, Duration::from_secs(5));
        let get_value2 = cache.get_timeout(key, Duration::from_secs(5));
        cache.put(key, value);
        assert_eq!(get_value1.await, Some(value));
        assert_eq!(get_value2.await, Some(value));
    }

    #[tokio::test]
    async fn get_times_out() {
        let cache = WaitCache::<u64, u64>::new(Duration::from_secs(30));
        let key = 1;
        assert_eq!(cache.get_timeout(key, Duration::from_millis(1)).await, None);
    }
}
