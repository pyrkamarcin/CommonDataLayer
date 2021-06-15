use lru_cache::LruCache;
use std::hash::Hash;
use std::future::Future;

pub struct DynamicCache<Key, Value, Fut>
    where Key: Eq + Hash + ToOwned<Owned = Key>,
    Fut: Future<Output = Result<Value, ()>>,
{
    on_missing: Box<dyn Fn(&Key) -> Fut>,
    inner: LruCache<Key, Value>,
}

impl<Key, Value, Fut> DynamicCache<Key, Value, Fut>
    where Key: Eq + Hash + ToOwned<Owned = Key>,
          Fut: Future<Output = Result<Value, ()>>, {
    pub fn new(capacity: usize, on_missing: Box<dyn Fn(&Key) -> Result<Value, ()>>) -> Self {
        Self {
            on_missing,
            inner: LruCache::new(capacity),
        }
    }

    pub async fn get(&mut self, key: impl AsRef<Key>) -> Result<&mut Value, ()> {
        let key = key.as_ref();

        if let Some(val) = self.inner.get_mut(key) {
            return Ok(val)
        }
        let val = (self.on_missing)(key).await?;
        self.inner.insert(key.to_owned(), val);
        self.inner.get_mut(key).ok_or(())
    }
}
