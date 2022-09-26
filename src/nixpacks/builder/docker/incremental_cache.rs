use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use super::dockerfile_generation::OutputDir;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tar::Archive;
use uuid::Uuid;

const INCREMENTAL_CACHE_DIR: &str = "incremental-cache";
const INCREMENTAL_CACHE_DOWNLOADS_DIR: &str = "downloads";
const INCREMENTAL_CACHE_UPLOADS_DIR: &str = "uploads";
const INCREMENTAL_CACHE_IMAGE_DIR: &str = "image";

#[derive(Default)]
pub struct IncrementalCache {}

#[derive(Serialize, Deserialize)]
struct ManifestItem {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

struct IncrementalCacheArchive {
    name: String,
    path: PathBuf,
}

pub struct IncrementalCacheConfig {
    pub downloads_dir: PathBuf,
    pub uploads_dir: PathBuf,
    pub image_dir: PathBuf,
    pub upload_server_access_token: String,
}

impl IncrementalCacheConfig {
    pub fn create(out_dir: &OutputDir) -> Result<IncrementalCacheConfig> {
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

        let downloads_dir =
            incremental_cache_root.join(PathBuf::from(INCREMENTAL_CACHE_DOWNLOADS_DIR));
        if fs::metadata(&downloads_dir)
            .context("Check if incremental cache downloads dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&downloads_dir)?;
        }

        let uploads_dir = incremental_cache_root.join(PathBuf::from(INCREMENTAL_CACHE_UPLOADS_DIR));
        if fs::metadata(&uploads_dir)
            .context("Check if incremental cache uploads dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&uploads_dir)?;
        }

        fs::create_dir_all(&image_dir).context("Create incremental cache image dir")?;
        fs::create_dir_all(&downloads_dir)
            .context("Creating incremental-cache downloads directory")?;
        fs::create_dir_all(&uploads_dir).context("Creating incremental-cache uploads directory")?;

        Ok(IncrementalCacheConfig {
            downloads_dir,
            uploads_dir,
            image_dir,
            upload_server_access_token: Uuid::new_v4().to_string(),
        })
    }

    pub fn get_downloads_relative_path(&self, filename: &str) -> PathBuf {
        PathBuf::from(INCREMENTAL_CACHE_DIR)
            .join(PathBuf::from(INCREMENTAL_CACHE_DOWNLOADS_DIR))
            .join(PathBuf::from(filename))
    }
}

impl IncrementalCache {
    pub fn download_files(
        &self,
        tag: &str,
        incremental_cache_config: &IncrementalCacheConfig,
    ) -> Result<bool> {
        let image_file_path = incremental_cache_config.image_dir.join("oci-image.tar");

        if !self.pull_image(tag)? {
            return Ok(false);
        }

        self.save_image(tag, &image_file_path)?;
        let archives =
            self.extract_archives(&image_file_path, &incremental_cache_config.image_dir)?;

        for item in archives {
            let filename_parts: Vec<&str> = item.name.split(".tar.nixpacks-").collect();
            if filename_parts.is_empty() {
                continue;
            }

            let filename: &str = filename_parts[0];
            let to_path = incremental_cache_config
                .downloads_dir
                .join(format!("{}.tar", filename));
            fs::copy(item.path, to_path).context("Move tar file to incremental cache")?;
        }

        if fs::metadata(&incremental_cache_config.image_dir).is_ok() {
            fs::remove_dir_all(&incremental_cache_config.image_dir)?;
        }

        Ok(true)
    }

    pub fn create_image(
        &self,
        incremental_cache_config: &IncrementalCacheConfig,
        tag: &str,
    ) -> Result<()> {
        let tar_file_path = &incremental_cache_config
            .uploads_dir
            .join(PathBuf::from("nixpacks-cached-dirs.tar"));

        if fs::metadata(tar_file_path).is_err() {
            return Ok(());
        }

        // There are three options to create a filesystem image that contains only tar files
        // #1 Use a Rust crate to create the image: 30+ seconds in a sample test, Also no clear winner Crate for creating OCI image
        // #2 Create minimal Dockerfile: 6 seconds in a sample test
        // #3 Use Docker import: Provide 3 seconds in a sample test
        let mut docker_import_cmd = Command::new("docker");
        docker_import_cmd
            .arg("import")
            .arg(&tar_file_path.display().to_string())
            .arg(&tag);

        let result = docker_import_cmd
            .spawn()?
            .wait()
            .context("Create incremental cache image")?;

        if !result.success() {
            bail!("Creating incremental cache image failed")
        }

        println!("Incremental cache image created: {}", &tag);
        Ok(())
    }

    fn pull_image(&self, tag: &str) -> Result<bool> {
        if tag.starts_with("https://") || tag.starts_with("http://") {
            bail!("Invalid image tag, should not start with https or http")
        }

        let mut docker_pull_cmd = Command::new("docker");

        docker_pull_cmd.arg("pull").arg(&tag);

        let result = docker_pull_cmd
            .spawn()?
            .wait()
            .context("Pull incremental cache image")?;

        Ok(result.success())
    }

    fn save_image(&self, tag: &str, tar_file_path: &Path) -> Result<()> {
        let mut docker_save_cmd = Command::new("docker");
        docker_save_cmd
            .arg("save")
            .arg("-o")
            .arg(tar_file_path.display().to_string())
            .arg(&tag);

        let result = docker_save_cmd
            .spawn()?
            .wait()
            .context("Save incremental cache image")?;

        if !result.success() {
            bail!("Saving incremental cache image failed")
        }

        Ok(())
    }

    fn extract_archives(
        &self,
        file_path: &PathBuf,
        dest_dir: &PathBuf,
    ) -> Result<Vec<IncrementalCacheArchive>> {
        let file = File::open(file_path)?;
        let mut archive = Archive::new(file);
        archive.unpack(&dest_dir)?;

        let json_path = dest_dir.join("manifest.json");
        let json_str = fs::read_to_string(json_path).context("Read manifest.json")?;
        let value: Vec<ManifestItem> = serde_json::from_str(&json_str)?;

        if value.first().is_none() {
            Ok(vec![])
        } else {
            let mut archives: Vec<IncrementalCacheArchive> = vec![];

            for item in value {
                for layer_name in &item.layers {
                    let tar_file_path = dest_dir.join(layer_name);
                    let extract_to = dest_dir.join(layer_name.replace("/layer.tar", "/layer"));

                    fs::create_dir_all(&extract_to).context("Create extract-to dir")?;
                    let file = File::open(&tar_file_path)?;

                    let mut archive = Archive::new(file);
                    archive.unpack(&extract_to)?;

                    let mut found_files = fs::read_dir(&extract_to)
                        .context("Read files of extract-to dir")?
                        .filter_map(|path| {
                            path.ok()?
                                .file_name()
                                .to_str()
                                .map(std::string::ToString::to_string)
                        })
                        .map(|value| IncrementalCacheArchive {
                            name: value.clone(),
                            path: extract_to.join(value),
                        })
                        .collect::<Vec<_>>();

                    archives.append(&mut found_files);
                }
            }
            Ok(archives)
        }
    }
}
