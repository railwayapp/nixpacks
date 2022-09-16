use super::{dockerfile_generation::DockerfileGenerator, DockerBuilderOptions, ImageBuilder};
use crate::nixpacks::{
    builder::docker::{dockerfile_generation::OutputDir, file_server::FileServer},
    environment::Environment,
    files,
    logger::Logger,
    plan::BuildPlan,
};
use anyhow::{bail, Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, remove_dir_all, File},
    path::{Path, PathBuf},
    process::Command,
};
use tar::Archive;
use tempdir::TempDir;
use uuid::Uuid;

pub struct DockerImageBuilder {
    logger: Logger,
    options: DockerBuilderOptions,
}

fn get_output_dir(app_src: &str, options: &DockerBuilderOptions) -> Result<OutputDir> {
    if let Some(value) = &options.out_dir {
        OutputDir::new(value.into(), false)
    } else if options.current_dir {
        OutputDir::new(app_src.into(), false)
    } else {
        let tmp = TempDir::new("nixpacks").context("Creating a temp directory")?;
        OutputDir::new(tmp.into_path(), true)
    }
}

fn write_incremental_cache_dockerfile(dir_path: &PathBuf) -> Result<PathBuf> {
    let dockerfile_path = dir_path.join("Dockerfile");
    if fs::metadata(&dockerfile_path).is_ok() {
        fs::remove_file(&dockerfile_path).context("Remove old incremental cache dockerfile")?;
    }

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

    let dockerfile = format!("FROM alpine\n{}", paths);

    fs::write(dockerfile_path.clone(), dockerfile).context("Write incremental cache dockerfile")?;

    Ok(dockerfile_path)
}

fn build_incremental_cache_image(dir_path: &PathBuf, tag: String) -> Result<()> {
    let dockerfile_path = write_incremental_cache_dockerfile(dir_path)?;
    let mut docker_build_cmd = Command::new("docker");

    // Enable BuildKit for all builds
    docker_build_cmd.env("DOCKER_BUILDKIT", "1");

    docker_build_cmd
        .arg("build")
        .arg(&dir_path.display().to_string())
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

fn pull_incremental_cache_from_image(tag: &str) -> Result<()> {
    let mut docker_build_cmd = Command::new("docker");

    docker_build_cmd.arg("pull").arg(&tag);

    let result = docker_build_cmd
        .spawn()?
        .wait()
        .context("Pull incremental cache image")?;

    if !result.success() {
        bail!("Pulling incremental cache image failed")
    }

    Ok(())
}

fn save_incremental_cache_image_to_tar(tag: &str, file_path: &Path) -> Result<()> {
    let mut docker_save_cmd = Command::new("docker");
    docker_save_cmd
        .arg("save")
        .arg("-o")
        .arg(file_path.display().to_string())
        .arg(&tag);

    let result = docker_save_cmd
        .spawn()?
        .wait()
        .context("Pull incremental cache image")?;

    if !result.success() {
        bail!("Pulling incremental cache image failed")
    }

    Ok(())
}

fn download_incremental_cache_files(tag: &str, out_dir: &OutputDir) -> Result<()> {
    let dir_path = out_dir.get_absolute_path("incremental-cache-image");
    if fs::metadata(&dir_path)
        .context("Check if incremental cache image dir exists")
        .is_err()
    {
        fs::create_dir_all(&dir_path).context("Create incremental cache image dir")?;
    }

    let tar_file_path = dir_path.join("oci-image.tar");

    pull_incremental_cache_from_image(tag)?;
    save_incremental_cache_image_to_tar(tag, &tar_file_path)?;
    let archives = extract_incremental_cache_image(
        &tar_file_path,
        &out_dir.get_absolute_path("incremental-cache-image"),
    )?;

    for item in archives {
        let to_path = out_dir
            .get_absolute_path("incremental-cache")
            .join(item.name);
        fs::rename(item.path, to_path).context("Move tar file to incremental cahce")?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}
struct IncrementalCacheArchive {
    name: String,
    path: PathBuf,
}

fn extract_incremental_cache_image(
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
            println!("layer_name {}", layer_name);

            let extract_to = dest_dir.join(layer_name.replace("/layer.tar", "/layer"));
            println!("extract_to {}", extract_to.display());

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

impl ImageBuilder for DockerImageBuilder {
    fn create_image(&self, app_src: &str, plan: &BuildPlan, env: &Environment) -> Result<()> {
        let id = Uuid::new_v4();

        let output = get_output_dir(app_src, &self.options)?;
        let name = self.options.name.clone().unwrap_or_else(|| id.to_string());
        output.ensure_output_exists()?;

        let file_server_access_token = Uuid::new_v4().to_string();

        let dockerfile = plan
            .generate_dockerfile(&self.options, env, &output, &file_server_access_token)
            .context("Generating Dockerfile for plan")?;

        // If printing the Dockerfile, don't write anything to disk
        if self.options.print_dockerfile {
            println!("{}", dockerfile);
            return Ok(());
        }

        if self.options.incremental_cache_image.is_some() {
            let save_to = output.root.join(".nixpacks").join("incremental-cache");
            if fs::metadata(&save_to)
                .context("Check if incremental-cache dir exists")
                .is_err()
            {
                fs::create_dir_all(&save_to).context("Creating incremental-cache directory")?;
            }

            download_incremental_cache_files(
                &self.options.incremental_cache_image.clone().unwrap(),
                &output,
            )?;

            let file_receiver = FileServer::new(save_to, file_server_access_token);
            file_receiver.start();
        }

        println!("{}", plan.get_build_string()?);

        let start = plan.start_phase.clone().unwrap_or_default();
        if start.cmd.is_none() && !self.options.no_error_without_start {
            bail!("No start command could be found")
        }

        self.write_app(app_src, &output).context("Writing app")?;
        self.write_dockerfile(dockerfile, &output)
            .context("Writing Dockerfile")?;
        plan.write_supporting_files(&self.options, env, &output)
            .context("Writing supporting files")?;

        // Only build if the --out flag was not specified
        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = self.get_docker_build_cmd(plan, name.as_str(), &output)?;

            // Execute docker build
            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;
            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");
            println!("\nRun:");
            println!("  docker run -it {}", name);

            if self.options.incremental_cache_image.is_some() {
                println!("Building  incremental cache image!");
                build_incremental_cache_image(
                    &output.get_absolute_path("incremental-cache"),
                    self.options.incremental_cache_image.clone().unwrap(),
                )?;
            }

            if output.is_temp {
                remove_dir_all(output.root)?;
            }
        } else {
            println!("\nSaved output to:");
            println!("  {}", output.root.to_str().unwrap());
        }

        Ok(())
    }
}

impl DockerImageBuilder {
    pub fn new(logger: Logger, options: DockerBuilderOptions) -> DockerImageBuilder {
        DockerImageBuilder { logger, options }
    }

    fn get_docker_build_cmd(
        &self,
        plan: &BuildPlan,
        name: &str,
        output: &OutputDir,
    ) -> Result<Command> {
        let mut docker_build_cmd = Command::new("docker");

        if docker_build_cmd.output().is_err() {
            bail!("Please install Docker to build the app https://docs.docker.com/engine/install/")
        }

        // Enable BuildKit for all builds
        docker_build_cmd.env("DOCKER_BUILDKIT", "1");

        docker_build_cmd
            .arg("build")
            .arg(&output.root)
            .arg("-f")
            .arg(&output.get_absolute_path("Dockerfile"))
            .arg("-t")
            .arg(name);

        if self.options.verbose {
            docker_build_cmd.arg("--progress=plain");
        }

        if self.options.quiet {
            docker_build_cmd.arg("--quiet");
        }

        if self.options.no_cache {
            docker_build_cmd.arg("--no-cache");
        }

        if let Some(value) = &self.options.cache_from {
            docker_build_cmd.arg("--cache-from").arg(value);
        }

        if self.options.inline_cache {
            docker_build_cmd
                .arg("--build-arg")
                .arg("BUILDKIT_INLINE_CACHE=1");
        }

        // Add build environment variables
        for (name, value) in &plan.variables.clone().unwrap_or_default() {
            docker_build_cmd
                .arg("--build-arg")
                .arg(format!("{}={}", name, value));
        }

        // Add user defined tags and labels to the image
        for t in self.options.tags.clone() {
            docker_build_cmd.arg("-t").arg(t);
        }
        for l in self.options.labels.clone() {
            docker_build_cmd.arg("--label").arg(l);
        }
        for l in self.options.platform.clone() {
            docker_build_cmd.arg("--platform").arg(l);
        }

        Ok(docker_build_cmd)
    }

    fn write_app(&self, app_src: &str, output: &OutputDir) -> Result<()> {
        if output.is_temp {
            files::recursive_copy_dir(app_src, &output.root)
        } else {
            Ok(())
        }
    }

    fn write_dockerfile(&self, dockerfile: String, output: &OutputDir) -> Result<()> {
        let dockerfile_path = output.get_absolute_path("Dockerfile");
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile).context("Write Dockerfile")?;

        Ok(())
    }
}
