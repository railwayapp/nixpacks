use super::cache::sanitize_cache_key;

pub fn get_cache_mount(
    cache_key: &Option<String>,
    cache_directories: &Option<Vec<String>>,
) -> String {
    match (cache_key, cache_directories) {
        (Some(cache_key), Some(cache_directories)) => cache_directories
            .iter()
            .map(|dir| {
                let sanitized_dir = dir.replace('~', "/root");
                let sanitized_key = sanitize_cache_key(&format!("{}-{}", cache_key, sanitized_dir));
                format!(
                    "--mount=type=cache,id={},target={}",
                    sanitized_key, sanitized_dir
                )
            })
            .collect::<Vec<String>>()
            .join(" "),
        _ => "".to_string(),
    }
}

pub fn get_copy_command(files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        "".to_owned()
    } else {
        format!("COPY {} {}", files.join(" "), app_dir)
    }
}

pub fn get_copy_from_command(from: &str, files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        format!("COPY --from=0 {} {}", app_dir, app_dir)
    } else {
        format!(
            "COPY --from={} {} {}",
            from,
            files
                .iter()
                .map(|f| f.replace("./", app_dir))
                .collect::<Vec<_>>()
                .join(" "),
            app_dir
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_mount() {
        let cache_key = Some("cache_key".to_string());
        let cache_directories = Some(vec!["dir1".to_string(), "dir2".to_string()]);

        let expected = "--mount=type=cache,id=cache_key-dir1,target=dir1 --mount=type=cache,id=cache_key-dir2,target=dir2";
        let actual = get_cache_mount(&cache_key, &cache_directories);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_cache_mount_invalid_cache_key() {
        let cache_key = Some("my cache key".to_string());
        let cache_directories = Some(vec!["dir1".to_string(), "dir2".to_string()]);

        let expected = "--mount=type=cache,id=my-cache-key-dir1,target=dir1 --mount=type=cache,id=my-cache-key-dir2,target=dir2";
        let actual = get_cache_mount(&cache_key, &cache_directories);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_copy_command() {
        let files = vec!["file1".to_string(), "file2".to_string()];
        let app_dir = "app";

        assert_eq!("".to_owned(), get_copy_command(&[], app_dir));
        assert_eq!(
            format!("COPY {} {}", files.join(" "), app_dir),
            get_copy_command(&files, app_dir)
        );
    }

    #[test]
    fn test_get_copy_from_command() {
        let from = "0";
        let files = vec!["file1".to_string(), "file2".to_string()];
        let app_dir = "app";

        assert_eq!(
            format!("COPY --from=0 {} {}", app_dir, app_dir),
            get_copy_from_command(from, &[], app_dir)
        );
        assert_eq!(
            format!("COPY --from={} {} {}", from, files.join(" "), app_dir),
            get_copy_from_command(from, &files, app_dir)
        );
    }
}
