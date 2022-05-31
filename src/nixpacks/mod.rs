use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};
use tempdir::TempDir;
use uuid::Uuid;
use walkdir::WalkDir;
pub mod app;
pub mod environment;
pub mod images;
pub mod logger;
pub mod nix;
pub mod phase;
pub mod plan;

use crate::providers::Provider;

use self::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    logger::Logger,
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
    plan::BuildPlan,
};

const NIX_PACKS_VERSION: &str = env!("CARGO_PKG_VERSION");

// https://status.nixos.org/
static NIXPKGS_ARCHIVE: &str = "41cc1d5d9584103be4108c1815c350e07c807036";

#[derive(Debug)]
pub struct AppBuilderOptions {
    pub custom_build_cmd: Option<String>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub pin_pkgs: bool,
    pub out_dir: Option<String>,
    pub plan_path: Option<String>,
    pub tags: Vec<String>,
    pub labels: Vec<String>,
    pub quiet: bool,
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
            labels: Vec::new(),
            quiet: false,
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
        let start_phase = self.get_start_phase().context("Generating start phase")?;
        let variables = self.get_variables().context("Getting plan variables")?;
        let static_assets = self
            .get_static_assets()
            .context("Getting provider assets")?;

        let plan = BuildPlan {
            version: Some(NIX_PACKS_VERSION.to_string()),
            setup: Some(setup_phase),
            install: Some(install_phase),
            build: Some(build_phase),
            start: Some(start_phase),
            variables: Some(variables),
            static_assets: Some(static_assets),
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

        let dir = match &self.options.out_dir {
            Some(dir) => dir.into(),
            None => {
                let tmp = TempDir::new("nixpacks").context("Creating a temp directory")?;
                tmp.into_path()
            }
        };
        let dir_path_str = dir.to_str().context("Invalid temp directory path")?;

        // Copy files into temp directory
        Self::recursive_copy_dir(&self.app.source, &dir)?;

        AppBuilder::write_build_plan(plan, dir_path_str).context("Writing build plan")?;
        self.logger.log_step("Building image with Docker");

        let name = self.name.clone().unwrap_or_else(|| id.to_string());

        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = Command::new("docker");
            docker_build_cmd
                .arg("build")
                .arg(dir)
                .arg("-t")
                .arg(name.clone());

            if self.options.quiet {
                docker_build_cmd.arg("--quiet");
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

            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;

            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");

            println!("\nRun:");
            println!("  docker run -it {}", name);
        } else {
            println!("\nSaved output to:");
            println!("  {}", dir_path_str);
        };

        Ok(())
    }

    fn recursive_copy_dir<T: AsRef<Path>, Q: AsRef<Path>>(source: T, dest: Q) -> Result<()> {
        let walker = WalkDir::new(&source).follow_links(true);
        for entry in walker {
            let entry = entry?;

            let from = entry.path();
            let to = dest.as_ref().join(from.strip_prefix(&source)?);

            // create directories
            if entry.file_type().is_dir() {
                if let Err(e) = fs::create_dir(to) {
                    match e.kind() {
                        io::ErrorKind::AlreadyExists => {}
                        _ => return Err(e.into()),
                    }
                }
            }
            // copy files
            else if entry.file_type().is_file() {
                fs::copy(from, to)?;
            }
        }
        Ok(())
    }

    fn get_setup_phase(&self) -> Result<SetupPhase> {
        let mut setup_phase: SetupPhase = match self.provider {
            Some(provider) => provider
                .setup(self.app, self.environment)?
                .unwrap_or_default(),
            None => SetupPhase::default(),
        };

        let env_var_pkgs = self
            .environment
            .get_config_variable("PKGS")
            .map(|pkg_string| pkg_string.split(' ').map(Pkg::new).collect::<Vec<_>>())
            .unwrap_or_default();

        // Add custom user packages
        let mut pkgs = [self.options.custom_pkgs.clone(), env_var_pkgs].concat();
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

        let env_build_cmd = self.environment.get_config_variable("BUILD_CMD").cloned();

        // Build command priority
        // - custom build command
        // - environment variable
        // - provider
        build_phase.cmd = self
            .options
            .custom_build_cmd
            .clone()
            .or(env_build_cmd)
            .or(build_phase.cmd);

        Ok(build_phase)
    }

    fn get_start_phase(&self) -> Result<StartPhase> {
        let procfile_cmd = self.parse_procfile()?;

        let mut start_phase = match self.provider {
            Some(provider) => provider
                .start(self.app, self.environment)?
                .unwrap_or_default(),
            None => StartPhase::default(),
        };

        let env_start_cmd = self.environment.get_config_variable("START_CMD").cloned();

        // Start command priority
        // - custom start command
        // - environment variable
        // - procfile
        // - provider
        start_phase.cmd = self
            .options
            .custom_start_cmd
            .clone()
            .or_else(|| env_start_cmd.or_else(|| procfile_cmd.or(start_phase.cmd)));

        // Allow the user to override the run image with an environment variable
        if let Some(env_run_image) = self.environment.get_config_variable("RUN_IMAGE") {
            // If the env var is "falsy", then unset the run image on the start phase
            start_phase.run_image = match env_run_image.as_str() {
                "0" | "false" => None,
                img if img.is_empty() => None,
                img => Some(img.to_owned()),
            };
        }

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

    fn get_static_assets(&self) -> Result<StaticAssets> {
        let static_assets = match self.provider {
            Some(provider) => provider
                .static_assets(self.app, self.environment)?
                .unwrap_or_default(),
            None => StaticAssets::new(),
        };

        Ok(static_assets)
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

        if let Some(assets) = &plan.static_assets {
            if !assets.is_empty() {
                let static_assets_path = PathBuf::from(dest).join(PathBuf::from("assets"));
                fs::create_dir_all(&static_assets_path).context("Creating static assets folder")?;

                for (name, content) in assets {
                    let path = Path::new(&static_assets_path).join(name);
                    let parent = path.parent().unwrap();
                    fs::create_dir_all(parent)
                        .context(format!("Creating parent directory for {}", name))?;
                    let mut file =
                        File::create(path).context(format!("Creating asset file for {name}"))?;
                    file.write_all(content.as_bytes())
                        .context(format!("Writing asset {name}"))?;
                }
            }
        }

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
        let assets_dir = app::ASSETS_DIR;

        let setup_phase = plan.setup.clone().unwrap_or_default();
        let install_phase = plan.install.clone().unwrap_or_default();
        let build_phase = plan.build.clone().unwrap_or_default();
        let start_phase = plan.start.clone().unwrap_or_default();
        let variables = plan.variables.clone().unwrap_or_default();
        let static_assets = plan.static_assets.clone().unwrap_or_default();

        // -- Variables
        let args_string = if !variables.is_empty() {
            format!(
                "ARG {}\nENV {}",
                // Pull the variables in from docker `--build-arg`
                variables
                    .iter()
                    .map(|var| var.0.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                // Make the variables available at runtime
                variables
                    .iter()
                    .map(|var| format!("{}=${}", var.0, var.0))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            "".to_string()
        };

        // -- Setup
        let mut setup_files: Vec<String> = vec!["environment.nix".to_string()];
        if let Some(mut setup_file_deps) = setup_phase.only_include_files {
            setup_files.append(&mut setup_file_deps);
        }
        let setup_copy_cmd = format!("COPY {} {}", setup_files.join(" "), app_dir);

        // -- Static Assets
        let assets_copy_cmd = if !static_assets.is_empty() {
            static_assets
                .into_keys()
                .map(|name| format!("COPY assets/{} {}{}", name, assets_dir, name))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            "".to_string()
        };

        // -- Install
        let install_cmd = install_phase
            .cmd
            .clone()
            .map(|cmd| format!("RUN {}", cmd))
            .unwrap_or_else(|| "".to_string());

        let path_env = if let Some(paths) = install_phase.paths {
            format!("ENV PATH {}:$PATH", paths.join(":"))
        } else {
            "".to_string()
        };

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

        let build_files = build_phase.only_include_files.unwrap_or_else(|| {
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
        let start_files = start_phase.only_include_files.clone();

        let run_image_setup = match start_phase.run_image {
            Some(run_image) => {
                // RUN true to prevent a Docker bug https://github.com/moby/moby/issues/37965#issuecomment-426853382
                formatdoc! {"
                FROM {run_image}
                WORKDIR {app_dir}
                COPY --from=0 /etc/ssl/certs /etc/ssl/certs
                RUN true
                {copy_cmd}
            ",
                    run_image=run_image,
                    app_dir=app_dir,
                    copy_cmd=get_copy_from_command("0", &start_files.unwrap_or_default(), app_dir)
                }
            }
            None => get_copy_command(
                // If no files specified and no run image, copy everything in /app/ over
                &start_files.unwrap_or_else(|| vec![".".to_string()]),
                app_dir,
            ),
        };

        let dockerfile = formatdoc! {"
          FROM {base_image}

          WORKDIR {app_dir}

          # Setup
          {setup_copy_cmd}
          {assets_copy_cmd}
          RUN nix-env -if environment.nix

          # Load environment variables
          {args_string}

          # Install
          {install_copy_cmd}
          {install_cmd}
          {path_env}

          # Build
          {build_copy_cmd}
          {build_cmd}

          # Start
          {run_image_setup}
          {start_cmd}
        ",
        base_image=setup_phase.base_image,
        setup_copy_cmd=setup_copy_cmd,
        args_string=args_string,
        install_copy_cmd=get_copy_command(&install_files, app_dir),
        install_cmd=install_cmd,
        path_env=path_env,
        build_copy_cmd=get_copy_command(&build_files, app_dir),
        build_cmd=build_cmd,
        run_image_setup=run_image_setup,
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

fn get_copy_from_command(from: &str, files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        format!(
            "
COPY --from=0 {} {}
RUN true
COPY --from=0 /assets /assets
",
            app_dir, app_dir
        )
    } else {
        format!(
            "COPY --from={} {} {}
            RUN true
            COPY --from={} /assets /assets",
            from,
            files
                .iter()
                .map(|f| f.replace("./", app_dir))
                .collect::<Vec<_>>()
                .join(" "),
            app_dir,
            from
        )
    }
}
