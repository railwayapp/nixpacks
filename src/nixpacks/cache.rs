pub type CacheKey = String;

pub trait Cache {
    fn get_cached_image(cache_key: CacheKey) -> Option<String>;
}

struct DockerCache {}

impl Cache for DockerCache {
    fn get_cached_image(cache_key: CacheKey) -> Option<String> {
        todo!()
    }
}
