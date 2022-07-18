use std::{collections::HashMap, fs};

use super::{BuildPlan, PlanGenerator};
use crate::{
    nixpacks::{
        app::{App, StaticAssets},
        environment::{Environment, EnvironmentVariables},
        nix::pkg::Pkg,
        phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
        NIX_PACKS_VERSION,
    },
    providers::Provider,
};
use anyhow::{bail, Context, Ok, Result};

// This line is automatically updated.
// Last Modified: 2022-07-13 08:25:28 UTC+0000
// https://github.com/NixOS/nixpkgs/commit/09066922296d9ef06bfadb937b2560524dd10785
static NIXPKGS_ARCHIVE: &str = "09066922296d9ef06bfadb937b2560524dd10785";

#[derive(Clone, Default, Debug)]
pub struct GeneratePlanOptions {
    pub custom_install_cmd: Option<Vec<String>>,
    pub custom_build_cmd: Option<Vec<String>>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub custom_libs: Vec<String>,
    pub custom_apt_pkgs: Vec<String>,
    pub pin_pkgs: bool,
    pub plan_path: Option<String>,
}

pub struct NixpacksBuildPlanGenerator<'a> {
    providers: Vec<&'a dyn Provider>,
    matched_provider: Option<&'a dyn Provider>,
    options: GeneratePlanOptions,
}

impl<'a> PlanGenerator for NixpacksBuildPlanGenerator<'a> {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan> {
        // If options.plan_path is specified, use that build plan
        if let Some(plan_path) = self.options.clone().plan_path {
            let plan_json = fs::read_to_string(plan_path).context("Reading build plan")?;
            let plan: BuildPlan =
                serde_json::from_str(&plan_json).context("Deserializing build plan")?;
            return Ok(plan);
        }

        self.detect(app, environment)?;

        let setup_phase = self
            .get_setup_phase(app, environment)
            .context("Getting setup phase")?;
        let install_phase = self
            .get_install_phase(app, environment)
            .context("Generating install phase")?;
        let build_phase = self
            .get_build_phase(app, environment)
            .context("Generating build phase")?;
        let start_phase = self
            .get_start_phase(app, environment)
            .context("Generating start phase")?;
        let variables = self
            .get_variables(app, environment)
            .context("Getting plan variables")?;
        let static_assets = self
            .get_static_assets(app, environment)
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
}

impl<'a> NixpacksBuildPlanGenerator<'a> {
    pub fn new(
        providers: Vec<&'a dyn Provider>,
        options: GeneratePlanOptions,
    ) -> NixpacksBuildPlanGenerator {
        NixpacksBuildPlanGenerator {
            providers,
            matched_provider: None,
            options,
        }
    }

    fn detect(&mut self, app: &App, environment: &Environment) -> Result<()> {
        for provider in self.providers.clone() {
            let matches = provider.detect(app, environment)?;
            if matches {
                self.matched_provider = Some(provider);
                break;
            }
        }

        Ok(())
    }

    fn get_setup_phase(&self, app: &App, environment: &Environment) -> Result<SetupPhase> {
        let mut setup_phase: SetupPhase = match self.matched_provider {
            Some(provider) => provider.setup(app, environment)?.unwrap_or_default(),
            None => SetupPhase::default(),
        };

        let env_var_pkgs = environment
            .get_config_variable("PKGS")
            .map(|pkg_string| pkg_string.split(' ').map(Pkg::new).collect::<Vec<_>>())
            .unwrap_or_default();

        // Add custom user packages
        let mut pkgs = [self.options.custom_pkgs.clone(), env_var_pkgs].concat();
        setup_phase.add_pkgs(&mut pkgs);

        let env_var_libs = environment
            .get_config_variable("LIBS")
            .map(|lib_string| {
                lib_string
                    .split(' ')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        // Add custom user libraries
        let libs = [self.options.custom_libs.clone(), env_var_libs].concat();
        setup_phase.add_libraries(libs);

        let env_var_apt_pkgs = environment
            .get_config_variable("APT_PKGS")
            .map(|apt_pkgs_string| {
                apt_pkgs_string
                    .split(' ')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        // Add custom apt packages
        let apt_pkgs = [self.options.custom_apt_pkgs.clone(), env_var_apt_pkgs].concat();
        setup_phase.add_apt_pkgs(apt_pkgs);

        if self.options.pin_pkgs {
            setup_phase.set_archive(NIXPKGS_ARCHIVE.to_string())
        }

        Ok(setup_phase)
    }

    fn get_install_phase(&self, app: &App, environment: &Environment) -> Result<InstallPhase> {
        let mut install_phase = match self.matched_provider {
            Some(provider) => provider.install(app, environment)?.unwrap_or_default(),
            None => InstallPhase::default(),
        };

        let mut env_install_cmd = None;
        if let Some(install_cmd) = environment.get_config_variable("INSTALL_CMD") {
            env_install_cmd = Some(vec![install_cmd]);
        }

        if let Some(install_cache_dirs) = environment.get_config_variable("INSTALL_CACHE_DIRS") {
            let custom_install_cache_dirs = install_cache_dirs
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            install_phase.cache_directories = match install_phase.cache_directories {
                Some(dirs) => Some([dirs, custom_install_cache_dirs].concat()),
                None => Some(custom_install_cache_dirs),
            }
        }

        // Start command priority
        // - custom install command
        // - environment variable
        // - provider

        install_phase.cmds = self
            .options
            .custom_install_cmd
            .clone()
            .or(env_install_cmd)
            .or(install_phase.cmds);

        Ok(install_phase)
    }

    fn get_build_phase(&self, app: &App, environment: &Environment) -> Result<BuildPhase> {
        let mut build_phase = match self.matched_provider {
            Some(provider) => provider.build(app, environment)?.unwrap_or_default(),
            None => BuildPhase::default(),
        };

        let mut env_build_cmd = None;
        if let Some(build_cmd) = environment.get_config_variable("BUILD_CMD") {
            env_build_cmd = Some(vec![build_cmd]);
        }

        if let Some(build_cache_dirs) = environment.get_config_variable("BUILD_CACHE_DIRS") {
            let custom_build_cache_dirs = build_cache_dirs
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            build_phase.cache_directories = match build_phase.cache_directories {
                Some(dirs) => Some([dirs, custom_build_cache_dirs].concat()),
                None => Some(custom_build_cache_dirs),
            }
        }

        // Build command priority
        // - custom build command
        // - environment variable
        // - provider
        build_phase.cmds = self
            .options
            .custom_build_cmd
            .clone()
            .or(env_build_cmd)
            .or(build_phase.cmds);

        // Release process type
        if let Some(release_cmd) = self.get_procfile_release_cmd(app)? {
            build_phase
                .cmds
                .clone()
                .unwrap_or_default()
                .push(release_cmd);
        }
        Ok(build_phase)
    }

    fn get_start_phase(&self, app: &App, environment: &Environment) -> Result<StartPhase> {
        let procfile_cmd = self.get_procfile_start_cmd(app)?;

        let mut start_phase = match self.matched_provider {
            Some(provider) => provider.start(app, environment)?.unwrap_or_default(),
            None => StartPhase::default(),
        };

        let env_start_cmd = environment.get_config_variable("START_CMD");

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
        if let Some(env_run_image) = environment.get_config_variable("RUN_IMAGE") {
            // If the env var is "falsy", then unset the run image on the start phase
            start_phase.run_image = match env_run_image.as_str() {
                "0" | "false" => None,
                img if img.is_empty() => None,
                img => Some(img.to_owned()),
            };
        }

        Ok(start_phase)
    }

    fn get_variables(&self, app: &App, environment: &Environment) -> Result<EnvironmentVariables> {
        // Get a copy of the variables in the environment
        let variables = Environment::clone_variables(environment);

        let new_variables = match self.matched_provider {
            Some(provider) => {
                // Merge provider variables
                let provider_variables = provider
                    .environment_variables(app, environment)?
                    .unwrap_or_default();
                provider_variables.into_iter().chain(variables).collect()
            }
            None => variables,
        };

        Ok(new_variables)
    }

    fn get_static_assets(&self, app: &App, environment: &Environment) -> Result<StaticAssets> {
        let static_assets = match self.matched_provider {
            Some(provider) => provider
                .static_assets(app, environment)?
                .unwrap_or_default(),
            None => StaticAssets::new(),
        };

        Ok(static_assets)
    }

    fn get_procfile_start_cmd(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let mut procfile: HashMap<String, String> =
                app.read_yaml("Procfile").context("Reading Procfile")?;
            procfile.remove("release");
            if procfile.len() > 1 {
                bail!("Procfile contains more than one process types. Please specify only one.");
            } else if procfile.is_empty() {
                Ok(None)
            } else {
                let process = Vec::from_iter(procfile.values())[0].to_string();
                Ok(Some(process))
            }
        } else {
            Ok(None)
        }
    }
    fn get_procfile_release_cmd(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let procfile: HashMap<String, String> =
                app.read_yaml("Procfile").context("Reading Procfile")?;
            if let Some(release) = procfile.get("release") {
                Ok(Some(release.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
