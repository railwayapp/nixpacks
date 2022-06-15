use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub type CacheKey = String;

pub trait Cache<T> {
    fn get_cached_value(&self, cache_key: &CacheKey) -> Result<Option<T>>;
    fn save_cached_value(&self, cache_key: CacheKey, value: T) -> Result<()>;
}

pub struct DockerCache {
    cache_location: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct CachedDockerImage {
    sha256: String,
}

impl DockerCache {
    pub fn new(cache_location: &str) -> Self {
        // Ensure that the cache_location exists
        let loc = PathBuf::from(cache_location);
        if !loc.is_dir() {
            fs::create_dir(loc.clone()).unwrap();
        }

        DockerCache {
            cache_location: loc,
        }
    }

    fn get_cache_value(&self, cache_key: &CacheKey) -> Result<Option<CachedDockerImage>> {
        let cache_path = self.get_cache_path(cache_key);
        if cache_path.is_file() {
            let cache_contents = fs::read_to_string(cache_path)?;
            let cache_value = serde_json::from_str::<CachedDockerImage>(cache_contents.as_str())?;
            Ok(Some(cache_value))
        } else {
            Ok(None)
        }
    }

    fn get_cache_path(&self, cache_key: &CacheKey) -> PathBuf {
        PathBuf::from(self.cache_location.clone()).join(cache_key)
    }
}

impl Cache<String> for DockerCache {
    fn get_cached_value(&self, cache_key: &CacheKey) -> Result<Option<String>> {
        // Look up /{cache_location}/{cache_key}
        match self.get_cache_value(cache_key)? {
            None => Ok(None),
            Some(cache_value) => {
                println!("CACHED VALUE: {:?}", cache_value);

                // If exists, get sha256 of cache_key Docker image
                // Compare hash to sha256 of cached image
                // TODO: Compare cache value

                Ok(Some(cache_key.clone()))
            }
        }
    }

    fn save_cached_value(&self, cache_key: CacheKey, value: String) -> Result<()> {
        // Get sha256 of cache_key Docker image
        let cache_value = CachedDockerImage {
            sha256: value.clone(),
        };

        // Save to /{cache_location}/{cache_key}
        let cache_path = self.get_cache_path(&cache_key);

        println!("Caching {value} to {cache_key}");

        fs::write(cache_path, serde_json::to_string_pretty(&cache_value)?)?;

        Ok(())
    }
}
