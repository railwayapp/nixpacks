pub fn sanitize_cache_key(cache_key: String) -> String {
    cache_key.replace("/", "-").replace(" ", "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizing_cache_key() {
        assert_eq!(sanitize_cache_key("key".to_string()), "key".to_string());
        assert_eq!(
            sanitize_cache_key("s p a c e s".to_string()),
            "s-p-a-c-e-s".to_string()
        );
        assert_eq!(
            sanitize_cache_key("s/l/a/s/h/e/s".to_string()),
            "s-l-a-s-h-e-s".to_string()
        );
    }
}
