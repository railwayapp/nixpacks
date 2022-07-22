use super::{dockerfile_generation::DockerfileGenerator, DockerBuilderOptions, ImageBuilder};
use crate::nixpacks::{
    environment::Environment, files, logger::Logger, plan::BuildPlan, NIX_PACKS_VERSION,
};
use anyhow::{bail, Context, Ok, Result};

use std::{
    fs::{self, File},
    path::PathBuf,
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;

pub struct DockerImageBuilder {
    logger: Logger,
    options: DockerBuilderOptions,
}

impl ImageBuilder for DockerImageBuilder {
    fn create_image(&self, app_src: &str, plan: &BuildPlan, env: &Environment) -> Result<()> {
        let id = Uuid::new_v4();

        let dir = match &self.options.out_dir {
            Some(dir) => dir.into(),
            None => {
                let tmp = TempDir::new("nixpacks").context("Creating a temp directory")?;
                tmp.into_path()
            }
        };
        let dest = dir.to_str().context("Invalid temp directory path")?;
        let name = self.options.name.clone().unwrap_or_else(|| id.to_string());

        let dockerfile = plan
            .generate_dockerfile(&self.options, env)
            .context("Generating Dockerfile for plan")?;

        // If printing the Dockerfile, don't write anything to disk
        if self.options.print_dockerfile {
            println!("{dockerfile}");
            return Ok(());
        }

        self.logger
            .log_section(format!("Building (nixpacks v{})", NIX_PACKS_VERSION).as_str());
        let build_plan_string = serde_json::to_string_pretty(plan).unwrap();
        println!("{}", build_plan_string);

        self.write_app(app_src, dest).context("Writing app")?;
        self.write_dockerfile(dockerfile, dest)
            .context("Writing Dockerfile")?;
        plan.write_supporting_files(&self.options, env, dest)
            .context("Writing supporting files")?;

        // Only build if the --out flag was not specified
        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = self.get_docker_build_cmd(plan, name.as_str(), dest)?;

            // Execute docker build
            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;
            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");
            println!("\nRun:");
            println!("  docker run -it {}", name);
        } else {
            println!("\nSaved output to:");
            println!("  {}", dest);
        }

        Ok(())
    }
}

impl DockerImageBuilder {
    pub fn new(logger: Logger, options: DockerBuilderOptions) -> DockerImageBuilder {
        DockerImageBuilder { logger, options }
    }

    fn get_docker_build_cmd(&self, plan: &BuildPlan, name: &str, dest: &str) -> Result<Command> {
        let mut docker_build_cmd = Command::new("docker");

        if docker_build_cmd.output().is_err() {
            bail!("Please install Docker to build the app https://docs.docker.com/engine/install/")
        }

        // Enable BuildKit for all builds
        docker_build_cmd.env("DOCKER_BUILDKIT", "1");

        docker_build_cmd.arg("build").arg(dest).arg("-t").arg(name);

        if self.options.quiet {
            docker_build_cmd.arg("--quiet");
        }

        if self.options.no_cache {
            docker_build_cmd.arg("--no-cache");
        }

        // Add build environment variables
        for (name, value) in plan.variables.clone().unwrap_or_default().iter() {
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

    fn write_app(&self, app_src: &str, dest: &str) -> Result<()> {
        files::recursive_copy_dir(app_src, &dest)
    }

    fn write_dockerfile(&self, dockerfile: String, dest: &str) -> Result<()> {
        let dockerfile_path = PathBuf::from(dest).join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile)?;

        Ok(())
    }
}
