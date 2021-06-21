#![feature(trait_alias)]

use anyhow::Context;
use lru_cache::LruCache;
use std::hash::Hash;

#[async_trait::async_trait]
pub trait CacheSupplier<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    async fn retrieve(&self, key: Key) -> anyhow::Result<Value>;
}

pub struct DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    cache_supplier: Box<dyn CacheSupplier<Key, Value> + Send + Sync>,
    inner: LruCache<Key, Value>,
}

impl<Key, Value> DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    pub fn new(
        capacity: usize,
        on_missing: Box<dyn CacheSupplier<Key, Value> + Send + Sync>,
    ) -> Self {
        Self {
            cache_supplier: on_missing,
            inner: LruCache::new(capacity),
        }
    }

    pub async fn get(&mut self, key: Key) -> anyhow::Result<&mut Value> {
        if self.inner.contains_key(&key) {
            return Ok(self.inner.get_mut(&key).unwrap());
        }

        let val = self.cache_supplier.retrieve(key.to_owned()).await?;
        self.inner.insert(key.to_owned(), val);
        self.inner.get_mut(&key).context("Critical error")
    }
}
