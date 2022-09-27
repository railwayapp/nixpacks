use super::{
    cache::sanitize_cache_key, dockerfile_generation::OutputDir,
    incremental_cache::{IncrementalCacheDirs}, file_server::FileServerConfig,
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf};

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

pub fn get_copy_out_cached_dirs_command(
    cache_directories: &Option<Vec<String>>,
    file_server_config: Option<FileServerConfig>,
) -> Vec<String> {
    let container_dirs = cache_directories.clone().unwrap_or_default();
    if container_dirs.is_empty() || file_server_config.is_none(){
        return vec![];
    }

    let server_config = file_server_config.unwrap();
    let unique_value_per_build = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Get time since UNIX epoch")
        .as_millis();

    let first_cmd = vec!["tar cvf nixpacks-cached-dirs.tar --files-from /dev/null".to_string()];

    let cmds =  container_dirs .iter()
            .flat_map(|dir| {
                let sanitized_dir =dir.replace('~', "/root");
                let compressed_file_name =  format!("{}.tar.nixpacks-{}", sanitized_dir.replace('/', "%2f"), unique_value_per_build);
                vec![
                    format!("if [ -d \"{}\" ]; then tar -cf {} {}; tar -rf nixpacks-cached-dirs.tar  {}; fi", sanitized_dir, compressed_file_name, sanitized_dir, compressed_file_name),                    
                    format!(
                        "if [ -d \"{}\" ]; then rm -rf {}; fi",
                        sanitized_dir,  sanitized_dir
                    ),
                ]
            })
            .collect::<Vec<String>>();

    let last_cmd =  vec![
        format!(
            "curl -v -T nixpacks-cached-dirs.tar {} --header \"t:{}\" --retry 3 --retry-all-errors --fail",
            server_config.upload_url, server_config.access_token,
        )
    ];

    [first_cmd, cmds, last_cmd].concat()
}

struct CachedDirInfo {
    target_cache_dir: String,
    compressed_file_name: String,
    source_file_path: String,
}
pub fn get_copy_in_cached_dirs_command(
    output_dir: &OutputDir,
    cache_directories: &Option<Vec<String>>,
    incremental_cache_dirs: &IncrementalCacheDirs,
) -> Vec<String> {
    let dirs = &cache_directories.clone().unwrap_or_default();
    if dirs.is_empty() {
        return vec![];
    }

    dirs.iter()
        .filter_map(|dir| {
            let target_cache_dir = dir.replace('~', "/root");
            let compressed_file_name = format!("{}.tar", target_cache_dir.replace('/', "%2f"));

            let source_file_path = incremental_cache_dirs
                .downloads_dir
                .join(PathBuf::from(&compressed_file_name));

            match fs::metadata(&source_file_path) {
                Ok(_) => {
                    let source_file_relative_path = output_dir.get_relative_path(
                        incremental_cache_dirs.get_downloads_relative_path(&compressed_file_name),
                    );

                    Some(CachedDirInfo {
                        target_cache_dir,
                        compressed_file_name,
                        source_file_path: source_file_relative_path.display().to_string(),
                    })
                }
                _ => None,
            }
        })
        .flat_map(|info| {
            let path_components_count = info
                .target_cache_dir
                .split('/')
                .into_iter()
                .filter(|c| !c.is_empty())
                .count();

            vec![
                format!(
                    "COPY {} {}",
                    info.source_file_path, info.compressed_file_name
                ),
                format!(
                    "RUN mkdir -p {}; tar -xf {} -C {} --strip-components {}",
                    info.target_cache_dir,
                    info.compressed_file_name,
                    info.target_cache_dir,
                    path_components_count
                ),
            ]
        })
        .collect::<Vec<String>>()
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
