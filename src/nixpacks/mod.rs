use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
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
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
    plan::BuildPlan,
};

const NIX_PACKS_VERSION: &str = env!("CARGO_PKG_VERSION");

// https://status.nixos.org/
static NIXPKGS_ARCHIVE: &str = "30d3d79b7d3607d56546dd2a6b49e156ba0ec634";

// Debian 11
static BASE_IMAGE: &str = "debian:buster-slim";

#[derive(Debug)]
pub struct AppBuilderOptions {
    pub custom_build_cmd: Option<String>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub pin_pkgs: bool,
    pub out_dir: Option<String>,
    pub plan_path: Option<String>,
    pub tags: Vec<String>,
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
            tags: Vec::new(),
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

        let setup_phase = self.get_setup_phase().context("Getting setup phase")?;
        let install_phase = self
            .get_install_phase()
            .context("Generating install phase")?;
        let build_phase = self.get_build_phase().context("Generating build phase")?;
        let start_phase = self.get_start_cmd().context("Generating start phase")?;
        let variables = self.get_variables().context("Getting plan variables")?;

        let plan = BuildPlan {
            version: Some(NIX_PACKS_VERSION.to_string()),
            setup: Some(setup_phase),
            install: Some(install_phase),
            build: Some(build_phase),
            start: Some(start_phase),
            variables: Some(variables),
        };

        Ok(plan)
    }

    pub fn build(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        self.logger
            .log_section(format!("Building (nixpacks v{})", NIX_PACKS_VERSION).as_str());

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

        println!("{}", plan.get_build_string());

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

        AppBuilder::write_build_plan(plan, dir.as_str()).context("Writing build plan")?;
        self.logger.log_step("Building image with Docker");

        let name = self.name.clone().unwrap_or_else(|| id.to_string());

        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = Command::new("docker");
            docker_build_cmd
                .arg("build")
                .arg(dir)
                .arg("-t")
                .arg(name.clone());

            // Add build environment variables
            for (name, value) in plan.variables.clone().unwrap_or_default().iter() {
                docker_build_cmd
                    .arg("--build-arg")
                    .arg(format!("{}={}", name, value));
            }

            // Add user defined tags to the image
            for t in self.options.tags.clone() {
                docker_build_cmd.arg("-t").arg(t);
            }

            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;

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
        let mut setup_phase: SetupPhase = match self.provider {
            Some(provider) => provider
                .setup(self.app, self.environment)?
                .unwrap_or_default(),
            None => SetupPhase::default(),
        };

        // Add custom user packages
        let mut pkgs = self.options.custom_pkgs.clone();
        setup_phase.add_pkgs(&mut pkgs);

        if self.options.pin_pkgs {
            setup_phase.set_archive(NIXPKGS_ARCHIVE.to_string())
        }

        Ok(setup_phase)
    }

    fn get_install_phase(&self) -> Result<InstallPhase> {
        let install_phase = match self.provider {
            Some(provider) => provider
                .install(self.app, self.environment)?
                .unwrap_or_default(),
            None => InstallPhase::default(),
        };

        Ok(install_phase)
    }

    fn get_build_phase(&self) -> Result<BuildPhase> {
        let mut build_phase = match self.provider {
            Some(provider) => provider
                .build(self.app, self.environment)?
                .unwrap_or_default(),
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
            Some(provider) => provider
                .start(self.app, self.environment)?
                .unwrap_or_default(),
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
            .or_else(|| procfile_cmd.or(start_phase.cmd));

        Ok(start_phase)
    }

    fn get_variables(&self) -> Result<EnvironmentVariables> {
        // Get a copy of the variables in the environment
        let variables = Environment::clone_variables(self.environment);

        let new_variables = match self.provider {
            Some(provider) => {
                // Merge provider variables
                let provider_variables = provider
                    .environment_variables(self.app, self.environment)?
                    .unwrap_or_default();
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
        let setup_phase = plan.setup.clone().unwrap_or_default();

        let nixpkgs = setup_phase
            .pkgs
            .iter()
            .map(|p| p.to_nix_string())
            .collect::<Vec<String>>()
            .join(" ");

        let nix_archive = setup_phase.archive.clone();
        let pkg_import = match nix_archive {
            Some(archive) => format!(
                "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
                archive
            ),
            None => "import <nixpkgs>".to_string(),
        };

        let mut overlays: Vec<String> = Vec::new();
        for pkg in &setup_phase.pkgs {
            if let Some(overlay) = &pkg.overlay {
                overlays.push(overlay.to_string());
            }
        }
        let overlays_string = overlays
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
        overlays=overlays_string};

        Ok(nix_expression)
    }

    pub fn gen_dockerfile(plan: &BuildPlan) -> Result<String> {
        let app_dir = "/app/";

        let setup_phase = plan.setup.clone().unwrap_or_default();
        let install_phase = plan.install.clone().unwrap_or_default();
        let build_phase = plan.build.clone().unwrap_or_default();
        let start_phase = plan.start.clone().unwrap_or_default();
        let variables = plan.variables.clone().unwrap_or_default();

        // -- Variables
        let args_string = format!(
            "ARG {}",
            variables
                .iter()
                .map(|var| var.0.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        // -- Setup
        let mut setup_files: Vec<String> = vec!["environment.nix".to_string()];
        if let Some(mut setup_file_deps) = setup_phase.only_include_files {
            setup_files.append(&mut setup_file_deps);
        }
        let setup_copy_cmd = format!("COPY {} {}", setup_files.join(" "), app_dir);

        // -- Install
        let install_cmd = install_phase
            .cmd
            .clone()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());

        // Files to copy for install phase
        // If none specified, copy over the entire app
        let install_files = install_phase
            .only_include_files
            .clone()
            .unwrap_or_else(|| vec![".".to_string()]);

        // -- Build
        let build_cmd = build_phase
            .cmd
            .clone()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());

        let build_files = build_phase.only_include_files.clone().unwrap_or_else(|| {
            // Only copy over the entire app if we haven't already in the install phase
            if install_phase.only_include_files.is_none() {
                Vec::new()
            } else {
                vec![".".to_string()]
            }
        });

        // -- Start
        let start_cmd = start_phase
            .cmd
            .map(|cmd| format!("CMD {}", cmd))
            .unwrap_or_else(|| "".to_string());

        // If we haven't yet copied over the entire app, do that before starting
        let mut start_files: Vec<String> = Vec::new();
        if build_phase.only_include_files.is_some() {
            start_files.push(".".to_string());
        }

        let dockerfile = formatdoc! {"
          FROM {base_image}

          RUN apt-get update && apt-get -y upgrade \\
            && apt-get install --no-install-recommends -y locales curl xz-utils ca-certificates openssl \\
            && apt-get clean && rm -rf /var/lib/apt/lists/* \\
            && mkdir -m 0755 /nix && mkdir -m 0755 /etc/nix && groupadd -r nixbld && chown root /nix \\
            && echo 'sandbox = false' > /etc/nix/nix.conf \\
            && for n in $(seq 1 10); do useradd -c \"Nix build user $n\" -d /var/empty -g nixbld -G nixbld -M -N -r -s \"$(command -v nologin)\" \"nixbld$n\"; done

          SHELL [\"/bin/bash\", \"-o\", \"pipefail\", \"-c\"]
          RUN set -o pipefail && curl -L https://nixos.org/nix/install | bash \\
              && /nix/var/nix/profiles/default/bin/nix-collect-garbage --delete-old

          ENV \\
            ENV=/etc/profile \\
            USER=root \\
            PATH=/nix/var/nix/profiles/default/bin:/nix/var/nix/profiles/default/sbin:/bin:/sbin:/usr/bin:/usr/sbin \\
            GIT_SSL_CAINFO=/etc/ssl/certs/ca-certificates.crt \\
            NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \\
            NIX_PATH=/nix/var/nix/profiles/per-user/root/channels

          RUN nix-channel --update

          RUN mkdir /app/
          WORKDIR /app/

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
        base_image=BASE_IMAGE,
        setup_copy_cmd=setup_copy_cmd,
        args_string=args_string,
        install_copy_cmd=get_copy_command(&install_files, app_dir),
        install_cmd=install_cmd,
        build_copy_cmd=get_copy_command(&build_files, app_dir),
        build_cmd=build_cmd,
        start_copy_cmd=get_copy_command(&start_files, app_dir),
        start_cmd=start_cmd};

        Ok(dockerfile)
    }
}

fn get_copy_command(files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        "".to_owned()
    } else {
        format!("COPY {} {}", files.join(" "), app_dir)
    }
}
