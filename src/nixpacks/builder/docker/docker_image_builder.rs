use super::{dockerfile_generation::DockerfileGenerator, DockerBuilderOptions, ImageBuilder};
use crate::nixpacks::{
    builder::docker::{
        dockerfile_generation::OutputDir,
        file_server::FileServer,
        incremental_cache::{IncrementalCache, IncrementalCacheDirs},
    },
    environment::Environment,
    files,
    logger::Logger,
    plan::BuildPlan,
};
use anyhow::{bail, Context, Ok, Result};
use std::{
    env,
    fs::{self, remove_dir_all, File},
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;

/// Builds Docker images from options, logging to stdout if the build is successful.
pub struct DockerImageBuilder {
    logger: Logger,
    options: DockerBuilderOptions,
}

/// Determine where to write project files and generated assets like Dockerfiles.
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

fn command_to_string(command: &Command) -> String {
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>();
    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        args.join(" ")
    )
}

use async_trait::async_trait;

#[async_trait]
impl ImageBuilder for DockerImageBuilder {
    /// Build a Docker image from a given BuildPlan and data from environment variables.
    async fn create_image(&self, app_src: &str, plan: &BuildPlan, env: &Environment) -> Result<()> {
        let id = Uuid::new_v4();

        let output = get_output_dir(app_src, &self.options)?;
        let name = self.options.name.clone().unwrap_or_else(|| id.to_string());
        output.ensure_output_exists()?;

        let incremental_cache = IncrementalCache::default();
        let incremental_cache_dirs = IncrementalCacheDirs::new(&output);

        let file_server_config = if self.options.incremental_cache_image.is_some() {
            incremental_cache_dirs.create()?;

            let file_server = FileServer {};
            let config = file_server.start(&incremental_cache_dirs);
            Some(config)
        } else {
            None
        };

        let dockerfile = plan
            .generate_dockerfile(&self.options, env, &output, file_server_config)
            .context("Generating Dockerfile for plan")?;

        // If printing the Dockerfile, don't write anything to disk
        if self.options.print_dockerfile {
            println!("{dockerfile}");
            return Ok(());
        }

        self.write_app(app_src, &output).context("Writing app")?;
        self.write_dockerfile(dockerfile, &output)
            .context("Writing Dockerfile")?;
        plan.write_supporting_files(&self.options, env, &output)
            .context("Writing supporting files")?;

        let mut docker_build_cmd = self.get_docker_build_cmd(plan, name.as_str(), &output)?;

        if self.options.out_dir.is_some() {
            let command_path = output.get_absolute_path("build.sh");
            File::create(command_path.clone()).context("Creating command.sh file")?;
            fs::write(command_path, command_to_string(&docker_build_cmd))
                .context("Write command")?;
        }

        // Only build if the --out flag was not specified
        if self.options.out_dir.is_none() {
            // Execute docker build
            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;
            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");
            println!("\nRun:");
            println!("  docker run -it {name}");

            if self.options.incremental_cache_image.is_some() {
                incremental_cache.create_image(
                    &incremental_cache_dirs,
                    &self.options.incremental_cache_image.clone().unwrap(),
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

    /// Generates the Docker command and arguments for building the project.
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
            .arg(output.get_absolute_path("Dockerfile"))
            .arg("-t")
            .arg(name);

        if self.options.verbose {
            docker_build_cmd.arg("--progress=plain");
        }

        if !self.options.add_host.is_empty() {
            for host in &self.options.add_host {
                docker_build_cmd.arg("--add-host").arg(host);
            }

            docker_build_cmd.arg("--network").arg("host");
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

        if let Some(value) = &self.options.docker_host {
            env::set_var("DOCKER_HOST", value);
        }

        if let Some(value) = &self.options.docker_tls_verify {
            if value == "1" {
                env::set_var("DOCKER_TLS_VERIFY", value);
            } else {
                env::remove_var("DOCKER_TLS_VERIFY"); // Clear the variable to disable TLS verification
            }
        }

        if let Some(value) = &self.options.docker_output {
            docker_build_cmd.arg("--output").arg(value);
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
                .arg(format!("{name}={value}"));
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

        if let Some(cpu_quota) = self.options.cpu_quota.clone() {
            docker_build_cmd.arg("--cpu-quota").arg(cpu_quota);
        }
        if let Some(memory) = self.options.memory.clone() {
            docker_build_cmd.arg("--memory").arg(memory);
        }

        Ok(docker_build_cmd)
    }

    /// Copies project files to temporary output dir, if that option was used.
    fn write_app(&self, app_src: &str, output: &OutputDir) -> Result<()> {
        if output.is_temp {
            files::recursive_copy_dir(app_src, &output.root)
        } else {
            Ok(())
        }
    }

    /// Writes the generated Dockerfile to the output dir.
    fn write_dockerfile(&self, dockerfile: String, output: &OutputDir) -> Result<()> {
        let dockerfile_path = output.get_absolute_path("Dockerfile");
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile).context("Write Dockerfile")?;

        Ok(())
    }
}
