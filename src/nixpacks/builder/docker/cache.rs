pub fn sanitize_cache_key(cache_key: &str) -> String {
    cache_key
        .chars()
        .filter(|x| !matches!(x, '.')) // remove dot from the string
        .map(|x| match x {
            ' ' => '-',
            _ => x,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizing_cache_key() {
        assert_eq!(sanitize_cache_key("key"), "key".to_string());
        assert_eq!(sanitize_cache_key("s p a c e s"), "s-p-a-c-e-s".to_string());
        assert_eq!(
            sanitize_cache_key("s/my-cache-key"),
            "s/my-cache-key".to_string()
        );
        assert_eq!(sanitize_cache_key("/.m2"), "/m2".to_string());
    }
}
