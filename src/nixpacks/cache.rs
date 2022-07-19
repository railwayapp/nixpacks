pub fn sanitize_cache_key(cache_key: &str) -> String {
    cache_key.replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizing_cache_key() {
        assert_eq!(sanitize_cache_key("key"), "key");
        assert_eq!(sanitize_cache_key("s p a c e s"), "s-p-a-c-e-s".to_string());
    }
}
