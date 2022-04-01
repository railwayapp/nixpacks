use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;
pub mod app;
pub mod logger;
pub mod plan;

use crate::providers::{Pkg, Provider};

use self::{app::App, logger::Logger, plan::BuildPlan};

static NIX_PACKS_VERSION: &str = "0.0.1";

// https://status.nixos.org/
static NIXPKGS_ARCHIVE: &str = "30d3d79b7d3607d56546dd2a6b49e156ba0ec634";

#[derive(Debug)]
pub struct AppBuilderOptions {
    pub custom_build_cmd: Option<String>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub pin_pkgs: bool,
}

impl AppBuilderOptions {
    pub fn empty() -> AppBuilderOptions {
        AppBuilderOptions {
            custom_build_cmd: None,
            custom_start_cmd: None,
            custom_pkgs: Vec::new(),
            pin_pkgs: false,
        }
    }
}

pub struct AppBuilder<'a> {
    name: Option<String>,
    app: &'a App,
    logger: &'a Logger,
    options: &'a AppBuilderOptions,
    provider: Option<&'a dyn Provider>,
}

impl<'a> AppBuilder<'a> {
    pub fn new(
        name: Option<String>,
        app: &'a App,
        logger: &'a Logger,
        options: &'a AppBuilderOptions,
    ) -> Result<AppBuilder<'a>> {
        Ok(AppBuilder {
            name,
            app,
            logger,
            options,
            provider: None,
        })
    }

    pub fn plan(&mut self, providers: Vec<&'a dyn Provider>) -> Result<BuildPlan> {
        // Load options from the best matching provider
        self.detect(providers).context("Detecting provider")?;

        let pkgs = self.get_pkgs().context("Getting packages")?;
        let install_cmd = self
            .get_install_cmd()
            .context("Generating install command")?;
        let build_cmd = self.get_build_cmd().context("Generating build command")?;
        let start_cmd = self.get_start_cmd().context("Generating start command")?;

        let plan = BuildPlan {
            version: NIX_PACKS_VERSION.to_string(),
            nixpkgs_archive: if self.options.pin_pkgs {
                Some(NIXPKGS_ARCHIVE.to_string())
            } else {
                None
            },
            pkgs,
            install_cmd,
            start_cmd,
            build_cmd,
        };

        Ok(plan)
    }

    pub fn build(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        self.logger.log_section("Building");
        let plan = self.plan(providers).context("Creating build plan")?;
        self.logger.log_step("Generated new build plan");

        self.do_build(&plan)
    }

    pub fn build_from_plan(&mut self, plan: &BuildPlan) -> Result<()> {
        self.logger.log_section("Building");
        self.logger.log_step("Building from existing plan");

        self.do_build(plan)
    }

    pub fn do_build(&mut self, plan: &BuildPlan) -> Result<()> {
        let id = Uuid::new_v4();
        let tmp_dir = TempDir::new("nixpacks").context("Creating a temp directory")?;

        self.logger.log_step("Copying source to tmp dir");

        let source = self.app.source.as_path().to_str().unwrap();
        let mut copy_cmd = Command::new("cp")
            .arg("-a")
            .arg(format!("{}/.", source))
            .arg(tmp_dir.path())
            .spawn()?;
        let copy_result = copy_cmd.wait().context("Copying app source to tmp dir")?;
        if !copy_result.success() {
            bail!("Copy failed")
        }

        self.logger.log_step("Writing build plan");
        AppBuilder::write_build_plan(plan, tmp_dir.path()).context("Writing build plan")?;

        self.logger.log_step("Building image");

        let name = self.name.clone().unwrap_or_else(|| id.to_string());

        let mut docker_build_cmd = Command::new("docker")
            .arg("build")
            .arg(tmp_dir.path())
            .arg("-t")
            .arg(name.clone())
            .spawn()?;

        let build_result = docker_build_cmd.wait().context("Building image")?;

        if !build_result.success() {
            bail!("Docker build failed")
        }

        self.logger.log_section("Successfully Built!");

        println!("\nRun:");
        println!("  docker run {}", name);

        Ok(())
    }

    fn get_pkgs(&self) -> Result<Vec<Pkg>> {
        let pkgs: Vec<Pkg> = match self.provider {
            Some(provider) => {
                let mut provider_pkgs = provider.pkgs(self.app);
                let mut pkgs = self.options.custom_pkgs.clone();
                pkgs.append(&mut provider_pkgs);
                pkgs
            }
            None => self.options.custom_pkgs.clone(),
        };

        Ok(pkgs)
    }

    fn get_install_cmd(&self) -> Result<Option<String>> {
        let install_cmd = match self.provider {
            Some(provider) => provider.install_cmd(self.app)?,
            None => None,
        };

        Ok(install_cmd)
    }

    fn get_build_cmd(&self) -> Result<Option<String>> {
        let suggested_build_cmd = match self.provider {
            Some(provider) => provider.suggested_build_cmd(self.app)?,
            None => None,
        };

        let build_cmd = self
            .options
            .custom_build_cmd
            .clone()
            .or(suggested_build_cmd);

        Ok(build_cmd)
    }

    fn get_start_cmd(&self) -> Result<Option<String>> {
        let procfile_cmd = self.parse_procfile()?;

        let suggested_start_cmd = match self.provider {
            Some(provider) => provider.suggested_start_command(self.app)?,
            None => None,
        };

        let start_cmd = self
            .options
            .custom_start_cmd
            .clone()
            .or(procfile_cmd)
            .or(suggested_start_cmd);

        Ok(start_cmd)
    }

    fn detect(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        for provider in providers {
            let matches = provider.detect(self.app)?;
            if matches {
                self.provider = Some(provider);
                break;
            }
        }

        Ok(())
    }

    fn parse_procfile(&self) -> Result<Option<String>> {
        if self.app.includes_file("Procfile") {
            let contents = self.app.read_file("Procfile")?;

            // Better error handling
            if contents.starts_with("web: ") {
                return Ok(Some(contents.replace("web: ", "").trim().to_string()));
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }

    pub fn write_build_plan(plan: &BuildPlan, dest: &Path) -> Result<()> {
        let nix_expression = AppBuilder::gen_nix(plan).context("Generating Nix expression")?;
        let dockerfile = AppBuilder::gen_dockerfile(plan).context("Generating Dockerfile")?;

        let nix_path = PathBuf::from(dest).join(PathBuf::from("environment.nix"));
        let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
        nix_file
            .write_all(nix_expression.as_bytes())
            .context("Unable to write Nix expression")?;

        let dockerfile_path = PathBuf::from(dest).join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile).context("Writing Dockerfile")?;

        Ok(())
    }

    pub fn gen_nix(plan: &BuildPlan) -> Result<String> {
        let nixpkgs = plan
            .pkgs
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<String>>()
            .join(" ");

        let nix_archive = plan.nixpkgs_archive.clone();
        let pkg_import = match nix_archive {
            Some(archive) => format!(
                "with import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
                archive
            ),
            None => "with import <nixpkgs>".to_string(),
        };

        let nix_expression = formatdoc! {"
           {pkg_import} {{ }}; [ {pkgs} ]
        ",
        pkg_import=pkg_import,
        pkgs=nixpkgs};

        Ok(nix_expression)
    }

    pub fn gen_dockerfile(plan: &BuildPlan) -> Result<String> {
        let install_cmd = plan
            .install_cmd
            .as_ref()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());
        let build_cmd = plan
            .build_cmd
            .as_ref()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());
        let start_cmd = plan
            .start_cmd
            .as_ref()
            .map(|cmd| format!("CMD {}", cmd))
            .unwrap_or_else(|| "".to_string());

        let dockerfile = formatdoc! {"
          FROM nixos/nix

          RUN nix-channel --update

          COPY . /app
          WORKDIR /app

          # Load Nix environment
          RUN nix-env -if environment.nix

          # Install
          {install_cmd}

          # Build
          {build_cmd}

          # Start
          {start_cmd}
        ",
        install_cmd=install_cmd,
        build_cmd=build_cmd,
        start_cmd=start_cmd};

        Ok(dockerfile)
    }
}
