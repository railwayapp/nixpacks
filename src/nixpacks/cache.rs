pub fn sanitize_cache_key(cache_key: String) -> String {
    cache_key
        .chars()
        .filter(|x| !matches!(x, '.')) // remove dot from the string
        .map(|x| match x {
            ' ' | '/' => '-',
            _ => x,
        })
        .collect()
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
        assert_eq!(sanitize_cache_key("/.m2".to_string()), "-m2".to_string());
    }
}
