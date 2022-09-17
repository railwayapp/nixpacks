use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use super::dockerfile_generation::OutputDir;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tar::Archive;

#[derive(Default)]
pub struct IncrementalCache {}

#[derive(Serialize, Deserialize)]
struct Manifest {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

struct IncrementalCacheArchive {
    name: String,
    path: PathBuf,
}

pub struct IncrementalCacheDirs {
    pub tar_archives_dir: PathBuf,
    pub image_dir: PathBuf,
}

impl IncrementalCache {
    pub fn download_files(&self, tag: &str, dirs: &IncrementalCacheDirs) -> Result<bool> {
        let image_file_path = dirs.image_dir.join("oci-image.tar");

        if !self.pull_image(tag)? {
            return Ok(false);
        }

        self.save_image(tag, &image_file_path)?;
        let archives = self.extract_archives(&image_file_path, &dirs.image_dir)?;

        for item in archives {
            let to_path = dirs.tar_archives_dir.join(item.name);
            fs::rename(item.path, to_path).context("Move tar file to incremental cahce")?;
        }

        Ok(true)
    }

    pub fn ensure_dirs_exists(&self, out_dir: &OutputDir) -> Result<IncrementalCacheDirs> {
        let save_to = out_dir.get_absolute_path("incremental-cache");
        if fs::metadata(&save_to)
            .context("Check if incremental-cache dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&save_to)?;
        }

        let dir_path = out_dir.get_absolute_path("incremental-cache-image");
        if fs::metadata(&dir_path)
            .context("Check if incremental cache image dir exists")
            .is_ok()
        {
            fs::remove_dir_all(&dir_path)?;
        }

        fs::create_dir_all(&save_to).context("Creating incremental-cache directory")?;
        fs::create_dir_all(&dir_path).context("Create incremental cache image dir")?;

        Ok(IncrementalCacheDirs {
            tar_archives_dir: save_to,
            image_dir: dir_path,
        })
    }

    fn write_dockerfile(&self, dir_path: &PathBuf) -> Result<PathBuf> {
        let dockerfile_path = dir_path.join("Dockerfile");
        let paths = fs::read_dir(&dir_path)
            .context("Read files at incremental cache dir")?
            .filter_map(|path| {
                path.ok()?
                    .file_name()
                    .to_str()
                    .map(|p| format!("COPY {} {}", p, p))
            })
            .collect::<Vec<_>>()
            .join("\n");

        let dockerfile = format!("FROM scratch\n{}", paths);

        fs::write(dockerfile_path.clone(), dockerfile)
            .context("Write incremental cache dockerfile")?;

        Ok(dockerfile_path)
    }

    pub fn create_image(&self, dirs: &IncrementalCacheDirs, tag: String) -> Result<()> {
        // ADR: writing a Dockerfile with minimal base image > copy tar files to > build the image 
        // Seems way faster than using some Rust crates to compose the OCI image file (1-2 minutes vs ~20 seconds) 
        // The overhead we need to take, is an extra layer to our final image with 5 MB of size. which is not that much compared with the average image size we deal with.
        let dockerfile_path = self.write_dockerfile(&dirs.tar_archives_dir)?;
        let mut docker_build_cmd = Command::new("docker");

        // Enable BuildKit for all builds
        docker_build_cmd.env("DOCKER_BUILDKIT", "1");

        docker_build_cmd
            .arg("build")
            .arg(&dirs.tar_archives_dir.display().to_string())
            .arg("-f")
            .arg(dockerfile_path.display().to_string())
            .arg("-t")
            .arg(tag);

        let result = docker_build_cmd
            .spawn()?
            .wait()
            .context("Build incremental cache image")?;

        if !result.success() {
            bail!("Building incremental cache image failed")
        }

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
        let value: Vec<Manifest> = serde_json::from_str(&json_str)?;

        if value.first().is_none() {
            Ok(vec![])
        } else {
            let mut archives: Vec<IncrementalCacheArchive> = vec![];
            for layer_name in value.first().unwrap().layers.iter().skip(1) {
                let tar_file_path = dest_dir.join(layer_name);
                let extract_to = dest_dir.join(layer_name.replace("/layer.tar", "/layer"));

                fs::create_dir_all(&extract_to).context("Create extract-to dir")?;
                let file = File::open(tar_file_path)?;

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
            Ok(archives)
        }
    }
}
