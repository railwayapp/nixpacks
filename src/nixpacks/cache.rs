use anyhow::Result;

pub type CacheKey = String;

pub trait Cache<T> {
    fn get_cached_value(&self, cache_key: &CacheKey) -> Result<Option<T>>;
    fn save_cached_value(&self, cache_key: CacheKey, value: T) -> Result<()>;
}
