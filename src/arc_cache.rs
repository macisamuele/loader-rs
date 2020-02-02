use cached::{Cached, UnboundCache};
use parking_lot::Mutex;
use std::{
    fmt::{Debug, Error, Formatter},
    hash::Hash,
    sync::Arc,
};

pub(crate) struct ThreadSafeCache<K: Clone + Eq + Hash, V>(Mutex<UnboundCache<K, Arc<V>>>);

impl<K: Clone + Eq + Hash, V> Debug for ThreadSafeCache<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let cache_lock = self.0.lock();
        write!(
            f,
            "ThreadSafeCache {{ cache_size: {}, cache_hits: {}, cache_misses: {} }}",
            cache_lock.cache_size(),
            cache_lock.cache_hits().unwrap_or(0),
            cache_lock.cache_misses().unwrap_or(0)
        )
    }
}

impl<K: Clone + Eq + Hash, V> Default for ThreadSafeCache<K, V> {
    fn default() -> Self {
        Self(Mutex::new(UnboundCache::new()))
    }
}

impl<K: Clone + Eq + Hash, V> ThreadSafeCache<K, V> {
    pub(crate) fn set(&self, key: &K, value: V) {
        self.set_arc(key, Arc::new(value));
    }

    pub(crate) fn set_arc(&self, key: &K, value: Arc<V>) {
        self.0.lock().cache_set(key.clone(), value);
    }

    pub(crate) fn get(&self, key: &K) -> Option<Arc<V>> {
        self.0.lock().cache_get(key).cloned()
    }
}
