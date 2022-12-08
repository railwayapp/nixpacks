use std::path::Path;

use super::cache::sanitize_cache_key;

pub fn get_cache_mount(
    cache_key: &Option<String>,
    cache_directories: &Option<Vec<String>>,
) -> String {
    match (cache_key, cache_directories) {
        (Some(cache_key), Some(cache_directories)) => cache_directories
            .iter()
            .map(|dir| {
                let mut sanitized_dir = dir.replace('~', "/root");
                let sanitized_key = sanitize_cache_key(&format!("{}-{}", cache_key, sanitized_dir));
                if !sanitized_dir.starts_with('/') {
                    sanitized_dir = format!("/app/{}", sanitized_dir);
                }
                format!(
                    "--mount=type=cache,id={},target={}",
                    sanitized_key, sanitized_dir
                )
            })
            .collect::<Vec<String>>()
            .join(" "),
        _ => String::new(),
    }
}

pub fn get_copy_commands(files: &[String], app_dir: &str) -> Vec<String> {
    if files.is_empty() {
        Vec::with_capacity(0)
    } else {
        files
            .iter()
            .map(|file| {
                let file_in_app_dir = Path::new(app_dir)
                    .join(file.strip_prefix("./").unwrap_or_default())
                    .join(file)
                    .display()
                    .to_string();

                format!("COPY {file} {file_in_app_dir}")
            })
            .collect()
    }
}

pub fn get_copy_from_commands(from: &str, files: &[String], app_dir: &str) -> Vec<String> {
    if files.is_empty() {
        vec![format!("COPY --from=0 {} {}", app_dir, app_dir)]
    } else {
        files
            .iter()
            .map(|file| {
                let file_in_app_dir = Path::new(app_dir)
                    .join(file.strip_prefix("./").unwrap_or_default())
                    .join(file)
                    .display()
                    .to_string();

                let file = if file.starts_with("./") {
                    &file_in_app_dir
                } else {
                    file
                };
                format!("COPY --from={from} {file} {file_in_app_dir}")
            })
            .collect()
    }
}

pub fn get_exec_command(command: &str) -> String {
    let params = command.replace('\"', "\\\"");

    format!("CMD [\"{}\"]", params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_mount() {
        let cache_key = Some("cache_key".to_string());
        let cache_directories = Some(vec!["dir1".to_string(), "dir2".to_string()]);

        let expected = "--mount=type=cache,id=cache_key-dir1,target=/app/dir1 --mount=type=cache,id=cache_key-dir2,target=/app/dir2";
        let actual = get_cache_mount(&cache_key, &cache_directories);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_cache_mount_invalid_cache_key() {
        let cache_key = Some("my cache key".to_string());
        let cache_directories = Some(vec!["dir1".to_string(), "dir2".to_string()]);

        let expected = "--mount=type=cache,id=my-cache-key-dir1,target=/app/dir1 --mount=type=cache,id=my-cache-key-dir2,target=/app/dir2";
        let actual = get_cache_mount(&cache_key, &cache_directories);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_copy_commands() {
        let files = vec!["file1".to_string(), "file2".to_string()];
        let app_dir = "app";

        assert_eq!(0, get_copy_commands(&[], app_dir).len());
        for (index, copy_command) in get_copy_commands(&files, app_dir).iter().enumerate() {
            assert_eq!(
                format!("COPY {} {}/{}", files[index], app_dir, files[index]),
                *copy_command
            );
        }
    }

    #[test]
    fn test_get_copy_from_command() {
        let from = "0";
        let files = vec!["file1".to_string(), "file2".to_string()];
        let app_dir = "app";

        assert_eq!(
            format!("COPY --from={from} {app_dir} {app_dir}"),
            get_copy_from_commands(from, &[], app_dir)[0]
        );
        for (index, copy_command) in get_copy_from_commands(from, &files, app_dir)
            .iter()
            .enumerate()
        {
            assert_eq!(
                format!(
                    "COPY --from={from} {} {}/{}",
                    files[index], app_dir, files[index]
                ),
                *copy_command
            );
        }
    }

    #[test]
    fn test_get_exec_cmd() {
        assert_eq!(
            "CMD [\"command1\"]".to_string(),
            get_exec_command("command1")
        );

        assert_eq!(
            "CMD [\"command1 command2\"]".to_string(),
            get_exec_command("command1 command2")
        );

        assert_eq!(
            "CMD [\"command1 command2 -l \\\"asdf\\\"\"]".to_string(),
            get_exec_command("command1 command2 -l \"asdf\"")
        );
    }
}
