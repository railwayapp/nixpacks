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
    fs::{self, remove_dir_all, File},
    process::Command,
};
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

impl ImageBuilder for DockerImageBuilder {
    fn create_image(&self, app_src: &str, plan: &BuildPlan, env: &Environment) -> Result<()> {
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
            println!("{}", dockerfile);
            return Ok(());
        }

        let phase_count = plan.phases.clone().map_or(0, |phases| phases.len());
        if phase_count > 0 {
            println!("{}", plan.get_build_string()?);

            let start = plan.start_phase.clone().unwrap_or_default();
            if start.cmd.is_none() && !self.options.no_error_without_start {
                bail!("No start command could be found")
            }
        } else {
            println!("\nNixpacks was unable to generate a build plan for this app.\nPlease check the documentation for supported languages: https://nixpacks.com");
            std::process::exit(1);
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
