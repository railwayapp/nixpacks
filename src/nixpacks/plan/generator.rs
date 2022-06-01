use super::{BuildPlan, PlanGenerator};
use crate::{
    nixpacks::{
        app::{App, StaticAssets},
        environment::{Environment, EnvironmentVariables},
        nix::pkg::Pkg,
        phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
    },
    providers::Provider,
};
use anyhow::{Context, Ok, Result};

// https://status.nixos.org/
static NIXPKGS_ARCHIVE: &str = "41cc1d5d9584103be4108c1815c350e07c807036";

const NIX_PACKS_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Default, Debug)]
pub struct GeneratePlanOptions {
    pub custom_build_cmd: Option<String>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
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

        if self.options.pin_pkgs {
            setup_phase.set_archive(NIXPKGS_ARCHIVE.to_string())
        }

        Ok(setup_phase)
    }

    fn get_install_phase(&self, app: &App, environment: &Environment) -> Result<InstallPhase> {
        let install_phase = match self.matched_provider {
            Some(provider) => provider.install(app, environment)?.unwrap_or_default(),
            None => InstallPhase::default(),
        };

        Ok(install_phase)
    }

    fn get_build_phase(&self, app: &App, environment: &Environment) -> Result<BuildPhase> {
        let mut build_phase = match self.matched_provider {
            Some(provider) => provider.build(app, environment)?.unwrap_or_default(),
            None => BuildPhase::default(),
        };

        let env_build_cmd = environment.get_config_variable("BUILD_CMD").cloned();

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

    fn get_start_phase(&self, app: &App, environment: &Environment) -> Result<StartPhase> {
        let procfile_cmd = self.parse_procfile(app)?;

        let mut start_phase = match self.matched_provider {
            Some(provider) => provider.start(app, environment)?.unwrap_or_default(),
            None => StartPhase::default(),
        };

        let env_start_cmd = environment.get_config_variable("START_CMD").cloned();

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

    fn parse_procfile(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let contents = app.read_file("Procfile")?;

            // Better error handling
            if contents.starts_with("web: ") {
                return Ok(Some(contents.replace("web: ", "").trim().to_string()));
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }
}
