use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

use super::{dockerfile_generation::OutputDir, file_server::FileServerConfig};
use anyhow::{bail, Context, Result};

const INCREMENTAL_CACHE_DIR: &str = "incremental-cache";
const INCREMENTAL_CACHE_UPLOADS_DIR: &str = "uploads";
const INCREMENTAL_CACHE_IMAGE_DIR: &str = "image";

#[derive(Default)]
pub struct IncrementalCache {}

#[derive(Default)]
pub struct IncrementalCacheDirs {
    pub uploads_dir: PathBuf,
    pub image_dir: PathBuf,
}

impl IncrementalCacheDirs {
    pub fn create(out_dir: &OutputDir) -> Result<IncrementalCacheDirs> {
        let incremental_cache_root = out_dir.get_absolute_path(INCREMENTAL_CACHE_DIR);

        if fs::metadata(&incremental_cache_root)
            .context("Check if incremental-cache dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&incremental_cache_root)?;
        }

        let image_dir = incremental_cache_root.join(PathBuf::from(INCREMENTAL_CACHE_IMAGE_DIR));
        if fs::metadata(&image_dir)
            .context("Check if incremental cache image dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&image_dir)?;
        }

        let uploads_dir = incremental_cache_root.join(PathBuf::from(INCREMENTAL_CACHE_UPLOADS_DIR));
        if fs::metadata(&uploads_dir)
            .context("Check if incremental cache uploads dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&uploads_dir)?;
        }

        fs::create_dir_all(&image_dir).context("Create incremental cache image dir")?;
        fs::create_dir_all(&uploads_dir).context("Creating incremental-cache uploads directory")?;

        Ok(IncrementalCacheDirs {
            uploads_dir,
            image_dir,
        })
    }
}

impl IncrementalCache {
    pub fn create_image(
        &self,
        incremental_cache_dirs: &IncrementalCacheDirs,
        tag: &str,
    ) -> Result<()> {
        let files = fs::read_dir(&incremental_cache_dirs.uploads_dir)?;

        // There are three options to create a filesystem image that contains only tar files
        // #1 Use a Rust crate to create the image: 30+ seconds in a sample test, Also no clear winner Crate for creating OCI image
        // #2 Create minimal Dockerfile: 6 seconds in a sample test
        // #3 Use Docker import: Provide 3 seconds in a sample test
        for f in files {
            let mut docker_import_cmd = Command::new("docker");
            docker_import_cmd.arg("import").arg(&f?.path()).arg(&tag);

            let result = docker_import_cmd
                .spawn()?
                .wait()
                .context("Create incremental cache image")?;

            if !result.success() {
                bail!("Creating incremental cache image failed")
            }
        }

        println!("Incremental cache image created: {}", &tag);
        Ok(())
    }

    pub fn is_image_exists(image_tag: &str) -> Result<bool> {
        // docker manifest inspect $IMGNAME:$IMGTAG

        let mut docker_inspect_cmd = Command::new("docker");
        docker_inspect_cmd
            .arg("manifest")
            .arg("inspect")
            .arg(&image_tag);

        let result = docker_inspect_cmd
            .spawn()?
            .wait()
            .context("Check incremental cache image exists in registry")?;

        Ok(result.success())
    }

    pub fn get_copy_to_image_command(
        cache_directories: &Option<Vec<String>>,
        incremental_cahge_image: &str,
    ) -> Result<Vec<String>> {
        let dirs = &cache_directories.clone().unwrap_or_default();
        if dirs.is_empty() || !IncrementalCache::is_image_exists(incremental_cahge_image)? {
            return Ok(vec![]);
        }

        let result = dirs
            .iter()
            .flat_map(|dir| {
                let target_cache_dir = dir.replace('~', "/root");
                vec![format!(
                    "COPY --from={} {} {}",
                    incremental_cahge_image, target_cache_dir, target_cache_dir
                )]
            })
            .collect::<Vec<String>>();

        Ok(result)
    }

    pub fn get_copy_from_image_command(
        cache_directories: &Option<Vec<String>>,
        file_server_config: Option<FileServerConfig>,
    ) -> Vec<String> {
        let container_dirs = cache_directories.clone().unwrap_or_default();
        if container_dirs.is_empty() || file_server_config.is_none() {
            return vec![];
        }

        let server_config = file_server_config.unwrap();
        container_dirs .iter()
            .flat_map(|dir| {
                let sanitized_dir =dir.replace('~', "/root");
                let compressed_file_name =  format!("{}.tar", sanitized_dir.replace('/', "%2f"));
                vec![
                    format!("if [ ! -d \"{sanitized_dir}\" ]; then make -p \"{sanitized_dir}\"; fi; tar -cf {compressed_file_name} {sanitized_dir};"),                    
                    format!(
                        "curl -v -T {} {} --header \"t:{}\" --retry 3 --retry-all-errors --fail",
                        compressed_file_name, server_config.upload_url, server_config.access_token,
                    ),
                    format!(
                        "rm -rf {}",
                        sanitized_dir
                    ),
                ]
            })
            .collect::<Vec<String>>()
    }
}
