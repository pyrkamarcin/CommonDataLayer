#![feature(trait_alias)]

use lru_cache::LruCache;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use anyhow::Context;

type BoxedProduceCacheItem<Key, Value> = Box<dyn ProduceCacheItem<Key, Value>>;
pub trait ProduceCacheItem<Key, Value> =
    Fn(&Key) -> Pin<Box<dyn Future<Output = anyhow::Result<Value>>>>;

pub struct DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    on_missing: BoxedProduceCacheItem<Key, Value>,
    inner: LruCache<Key, Value>,
}

impl<Key, Value> DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    pub fn new(capacity: usize, on_missing: impl ProduceCacheItem<Key, Value> + 'static) -> Self {
        Self {
            on_missing: Box::new(on_missing),
            inner: LruCache::new(capacity),
        }
    }

    pub async fn get(&mut self, key: impl AsRef<Key>) -> anyhow::Result<&mut Value> {
        let key = key.as_ref();

        if self.inner.contains_key(key) {
            return Ok(self.inner.get_mut(key).unwrap());
        }

        let val = (self.on_missing)(key).await?;
        self.inner.insert(key.to_owned(), val);
        self.inner.get_mut(key).context("Critical error")
    }
}
