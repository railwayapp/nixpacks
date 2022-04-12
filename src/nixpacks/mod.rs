use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
    num::IntErrorKind,
    path::PathBuf,
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;
pub mod app;
pub mod environment;
pub mod logger;
pub mod nix;
pub mod phase;
pub mod plan;

use crate::providers::Provider;

use self::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    logger::Logger,
    nix::{NixConfig, Pkg},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
    plan::BuildPlan,
};

static NIX_PACKS_VERSION: &str = "0.0.1";

// https://status.nixos.org/
static NIXPKGS_ARCHIVE: &str = "30d3d79b7d3607d56546dd2a6b49e156ba0ec634";

#[derive(Debug)]
pub struct AppBuilderOptions {
    pub custom_build_cmd: Option<String>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub pin_pkgs: bool,
    pub out_dir: Option<String>,
    pub plan_path: Option<String>,
}

impl AppBuilderOptions {
    pub fn empty() -> AppBuilderOptions {
        AppBuilderOptions {
            custom_build_cmd: None,
            custom_start_cmd: None,
            custom_pkgs: Vec::new(),
            pin_pkgs: false,
            out_dir: None,
            plan_path: None,
        }
    }
}

pub struct AppBuilder<'a> {
    name: Option<String>,
    app: &'a App,
    environment: &'a Environment,
    logger: &'a Logger,
    options: &'a AppBuilderOptions,
    provider: Option<&'a dyn Provider>,
}

impl<'a> AppBuilder<'a> {
    pub fn new(
        name: Option<String>,
        app: &'a App,
        environment: &'a Environment,
        logger: &'a Logger,
        options: &'a AppBuilderOptions,
    ) -> Result<AppBuilder<'a>> {
        Ok(AppBuilder {
            name,
            app,
            environment,
            logger,
            options,
            provider: None,
        })
    }

    pub fn plan(&mut self, providers: Vec<&'a dyn Provider>) -> Result<BuildPlan> {
        // Load options from the best matching provider
        self.detect(providers).context("Detecting provider")?;

        // let nix_config = self.get_nix_config().context("Getting packages")?;
        let setup_phase = self.get_setup_phase().context("Getting setup phase")?;
        let install_phase = self
            .get_install_phase()
            .context("Generating install phase")?;
        let build_phase = self.get_build_phase().context("Generating build phase")?;
        let start_phase = self.get_start_cmd().context("Generating start phase")?;
        let variables = self.get_variables().context("Getting plan variables")?;

        let plan = BuildPlan {
            version: NIX_PACKS_VERSION.to_string(),
            setup: setup_phase,
            install: install_phase,
            build: build_phase,
            start: start_phase,
            variables,
        };

        Ok(plan)
    }

    pub fn build(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        self.logger.log_section("Building");

        let plan = match &self.options.plan_path {
            Some(plan_path) => {
                self.logger.log_step("Building from existing plan");
                let plan_json = fs::read_to_string(plan_path).context("Reading build plan")?;
                let plan: BuildPlan =
                    serde_json::from_str(&plan_json).context("Deserializing build plan")?;
                plan
            }
            None => {
                self.logger.log_step("Generated new build plan");

                self.plan(providers).context("Creating build plan")?
            }
        };

        self.do_build(&plan)
    }

    pub fn do_build(&mut self, plan: &BuildPlan) -> Result<()> {
        let id = Uuid::new_v4();

        let dir: String = match &self.options.out_dir {
            Some(dir) => dir.clone(),
            None => {
                let tmp = TempDir::new("nixpacks").context("Creating a temp directory")?;
                let path = tmp.path().to_str().unwrap();
                path.to_string()
            }
        };

        self.logger.log_step("Copying source to tmp dir");

        let source = self.app.source.as_path().to_str().unwrap();
        let mut copy_cmd = Command::new("cp")
            .arg("-a")
            .arg(format!("{}/.", source))
            .arg(dir.clone())
            .spawn()?;
        let copy_result = copy_cmd.wait().context("Copying app source to tmp dir")?;
        if !copy_result.success() {
            bail!("Copy failed")
        }

        self.logger.log_step("Writing build plan");
        AppBuilder::write_build_plan(plan, dir.as_str()).context("Writing build plan")?;

        self.logger.log_step("Building image");

        let name = self.name.clone().unwrap_or_else(|| id.to_string());

        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = Command::new("docker")
                .arg("build")
                .arg(dir)
                .arg("-t")
                .arg(name.clone())
                .spawn()?;

            let build_result = docker_build_cmd.wait().context("Building image")?;

            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");

            println!("\nRun:");
            println!("  docker run -it {}", name);
        } else {
            println!("\nSaved output to:");
            println!("  {}", dir);
        };

        Ok(())
    }

    fn get_setup_phase(&self) -> Result<SetupPhase> {
        let base_setup_phase = SetupPhase::new(NixConfig::new(vec![Pkg::new("stdenv")]));

        let mut setup_phase: SetupPhase = match self.provider {
            Some(provider) => provider.setup(self.app, self.environment)?,
            None => base_setup_phase,
        };

        // Add custom user packages
        let mut pkgs = self.options.custom_pkgs.clone();
        setup_phase.nix_config.add_pkgs(&mut pkgs);

        if self.options.pin_pkgs {
            setup_phase
                .nix_config
                .set_archive(NIXPKGS_ARCHIVE.to_string())
        }

        Ok(setup_phase)
    }

    fn get_install_phase(&self) -> Result<InstallPhase> {
        let install_phase = match self.provider {
            Some(provider) => provider.install(self.app, self.environment)?,
            None => InstallPhase::default(),
        };

        Ok(install_phase)
    }

    fn get_build_phase(&self) -> Result<BuildPhase> {
        let mut build_phase = match self.provider {
            Some(provider) => provider.build(self.app, self.environment)?,
            None => BuildPhase::default(),
        };

        if let Some(custom_build_cmd) = self.options.custom_build_cmd.clone() {
            build_phase.cmd = Some(custom_build_cmd);
        }

        Ok(build_phase)
    }

    fn get_start_cmd(&self) -> Result<StartPhase> {
        let procfile_cmd = self.parse_procfile()?;

        let mut start_phase = match self.provider {
            Some(provider) => provider.start(self.app, self.environment)?,
            None => StartPhase::default(),
        };

        // Start command priority
        // - custom start command
        // - procfile
        // - provider
        start_phase.cmd = self
            .options
            .custom_start_cmd
            .clone()
            .or_else(|| procfile_cmd.or_else(|| start_phase.cmd));

        Ok(start_phase)
    }

    fn get_variables(&self) -> Result<EnvironmentVariables> {
        // Get a copy of the variables in the environment
        let variables = Environment::clone_variables(self.environment);

        let new_variables = match self.provider {
            Some(provider) => {
                // Merge provider variables
                let provider_variables =
                    provider.environment_variables(self.app, self.environment)?;
                provider_variables.into_iter().chain(variables).collect()
            }
            None => variables,
        };

        Ok(new_variables)
    }

    fn detect(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        for provider in providers {
            let matches = provider.detect(self.app, self.environment)?;
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

    pub fn write_build_plan(plan: &BuildPlan, dest: &str) -> Result<()> {
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
            .setup
            .nix_config
            .pkgs
            .iter()
            .map(|p| p.to_nix_string())
            .collect::<Vec<String>>()
            .join(" ");

        let nix_archive = plan.setup.nix_config.nixpkgs_archive.clone();
        let pkg_import = match nix_archive {
            Some(archive) => format!(
                "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
                archive
            ),
            None => "import <nixpkgs>".to_string(),
        };

        let overlays = plan
            .setup
            .nix_config
            .overlays
            .iter()
            .map(|url| format!("(import (builtins.fetchTarball \"{}\"))", url))
            .collect::<Vec<String>>()
            .join("\n");

        let nix_expression = formatdoc! {"
            {{ }}:

            let
              pkgs = {pkg_import} {{ 
                overlays = [
                  {overlays}
                ];
              }};
            in with pkgs;
            buildEnv {{
              name = \"env\";
              paths = [
                {pkgs}
              ];
            }}
        ",
        pkg_import=pkg_import,
        pkgs=nixpkgs,
        overlays=overlays};

        Ok(nix_expression)
    }

    pub fn gen_dockerfile(plan: &BuildPlan) -> Result<String> {
        let app_dir = "/app";

        // -- Variables
        let args_string = plan
            .variables
            .iter()
            .map(|var| format!("ENV {}='{}'", var.0, var.1))
            .collect::<Vec<String>>()
            .join("\n");

        // -- Setup
        let mut setup_files: Vec<String> = vec!["environment.nix".to_string()];
        setup_files.append(&mut plan.setup.file_dependencies.clone());
        let setup_copy_cmd = format!("COPY {} {}", setup_files.join(" "), app_dir);

        // Whether or not we have copied over the entire app yet (so we don't do it twice)
        let mut copied_app = false;

        // -- Install
        let install_cmd = plan
            .install
            .cmd
            .clone()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());

        // Files to copy for install phase
        // If none specified, copy over the entire app
        let mut install_files = plan.install.file_dependencies.clone();
        if install_files.len() == 0 {
            install_files.push(".".to_string());
            copied_app = true;
        }
        let install_copy_cmd = match !install_files.is_empty() {
            true => format!("COPY {} {}", install_files.join(" "), app_dir),
            false => "".to_owned(),
        };

        // -- Build
        let build_cmd = plan
            .build
            .cmd
            .clone()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());
        let mut build_files = plan.build.file_dependencies.clone();
        if !copied_app && build_files.is_empty() {
            build_files.push(".".to_string());
            copied_app = true;
        }
        let build_copy_cmd = match !build_files.is_empty() {
            true => format!("COPY {} {}", build_files.join(" "), app_dir),
            false => "".to_owned(),
        };

        // -- Start
        let start_cmd = plan
            .start
            .cmd
            .clone()
            .map(|cmd| format!("CMD {}", cmd))
            .unwrap_or_else(|| "".to_string());

        let mut start_files = plan.build.file_dependencies.clone();
        if !copied_app && start_files.is_empty() {
            start_files.push(".".to_string());
        }
        let start_copy_cmd = match !start_files.is_empty() {
            true => format!("COPY {} {}", start_files.join(" "), app_dir),
            false => "".to_owned(),
        };

        let dockerfile = formatdoc! {"
          FROM nixos/nix
          RUN nix-channel --update

          RUN mkdir /app
          WORKDIR /app

          # Setup
          {setup_copy_cmd}
          RUN nix-env -if environment.nix

          # Load environment variables
          {args_string}

          # Install
          {install_copy_cmd}
          {install_cmd}

          # Build
          {build_copy_cmd}
          {build_cmd}

          # Start
          {start_copy_cmd}
          {start_cmd}
        ",
        setup_copy_cmd=setup_copy_cmd,
        args_string=args_string,
        install_copy_cmd=install_copy_cmd,
        install_cmd=install_cmd,
        build_copy_cmd=build_copy_cmd,
        build_cmd=build_cmd,
        start_copy_cmd=start_copy_cmd,
        start_cmd=start_cmd};

        Ok(dockerfile)
    }
}
