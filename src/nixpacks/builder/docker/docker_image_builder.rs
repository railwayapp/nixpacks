use super::{dockerfile_generation::DockerfileGenerator, DockerBuilderOptions, ImageBuilder};
use crate::nixpacks::{
    builder::docker::{
        dockerfile_generation::OutputDir,
        file_server::FileServer,
        incremental_cache::{IncrementalCache, IncrementalCacheDirs},
    },
    clients::docker::Docker,
    environment::Environment,
    files,
    logger::Logger,
    plan::BuildPlan,
};
use anyhow::{bail, Context, Ok, Result};
use async_trait::async_trait;
use bollard::image::BuildImageOptions;
use futures_util::stream::StreamExt;
use std::{
    collections::HashMap,
    fs::{self, remove_dir_all, File},
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;

pub struct DockerImageBuilder {
    logger: Logger,
    options: DockerBuilderOptions,
    docker: Docker,
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

#[async_trait]
impl ImageBuilder for DockerImageBuilder {
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
            self.logger.log_section("Building");

            let res = self
                .execute_docker_build(plan, name.as_str(), &output)
                .await;

            if res.is_err() {
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
    pub fn new(
        logger: Logger,
        options: DockerBuilderOptions,
        docker: Docker,
    ) -> DockerImageBuilder {
        DockerImageBuilder {
            logger,
            options,
            docker,
        }
    }

    async fn execute_docker_build(
        &self,
        plan: &BuildPlan,
        name: &str,
        output: &OutputDir,
    ) -> Result<()> {
        // Set default variables for the build
        let mut vars = HashMap::from([("DOCKER_BUILDKIT".to_string(), "1".to_string())]);

        if self.options.inline_cache {
            vars.insert("BUILDKIT_INLINE_CACHE".to_string(), "1".to_string());
        }

        // Add build environment variables
        let build_vars: HashMap<_, _> = plan
            .variables
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect();

        // fold build_vars into vars
        vars.extend(build_vars);

        // Generate label map
        let labels: HashMap<_, _> = self
            .options
            .labels
            .iter()
            .map(|l| {
                let mut parts = l.splitn(2, '=');
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();
                (key.to_string(), value.to_string())
            })
            .into_iter()
            .collect();

        let mut stream = self.docker.client.build_image(
            BuildImageOptions {
                buildargs: vars.into(),
                cachefrom: vec![self.options.cache_from.clone().unwrap_or_default()],
                // Todo: need to support multiple tags?
                t: name.to_string(),
                dockerfile: output
                    .get_absolute_path("Dockerfile")
                    .to_str()
                    .unwrap()
                    .to_string(),
                q: self.options.quiet,
                // todo: need to support multiple platforms?
                // platform: self.options.platform,
                labels,
                nocache: self.options.no_cache,
                ..Default::default()
            },
            None,
            None,
        );

        while let Some(msg) = stream.next().await {
            println!("{:?}", msg);
        }
        return Ok(());
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
