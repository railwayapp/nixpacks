use super::{config::GeneratePlanConfig, BuildPlan, PlanGenerator};
use crate::{
    nixpacks::{app::App, environment::Environment, nix::pkg::Pkg},
    providers::Provider,
};
use anyhow::{Context, Ok, Result};
use std::collections::HashMap;

// This line is automatically updated.
// Last Modified: 2022-08-29 17:07:50 UTC+0000
// https://github.com/NixOS/nixpkgs/commit/0e304ff0d9db453a4b230e9386418fd974d5804a
pub const NIXPKGS_ARCHIVE: &str = "0e304ff0d9db453a4b230e9386418fd974d5804a";

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
    providers: &'a [&'a dyn Provider],
    matched_provider: Option<&'a dyn Provider>,
    config: GeneratePlanConfig,
}

impl<'a> PlanGenerator for NixpacksBuildPlanGenerator<'a> {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan> {
        // Match a specific provider
        self.detect(app, environment)?;

        // If the provider defines a build plan in the new format, use that
        let plan = self.get_build_plan(app, environment)?;

        Ok(plan)
    }
}

impl NixpacksBuildPlanGenerator<'_> {
    pub fn new<'a>(
        providers: &'a [&'a dyn Provider],
        config: GeneratePlanConfig,
    ) -> NixpacksBuildPlanGenerator<'a> {
        NixpacksBuildPlanGenerator {
            providers,
            matched_provider: None,
            config,
        }
    }

    /// Match a single provider from the given app and environment.
    fn detect(&mut self, app: &App, environment: &Environment) -> Result<()> {
        for &provider in self.providers {
            let matches = provider.detect(app, environment)?;
            if matches {
                self.matched_provider = Some(provider);
                break;
            }
        }

        Ok(())
    }

    /// Get a build plan from the provider and by applying the config from the environment
    fn get_build_plan(&self, app: &App, environment: &Environment) -> Result<BuildPlan> {
        // Merge the config from the CLI flags with the config from the environment variables
        // The CLI config takes precedence
        let config = GeneratePlanConfig::merge(
            &GeneratePlanConfig::from_environment(environment),
            &self.config,
        );

        let mut plan = BuildPlan::default();

        if let Some(provider) = self.matched_provider {
            if let Some(provider_build_plan) = provider.get_build_plan(app, environment)? {
                plan = provider_build_plan;
            }
        }

        // The Procfile start command has precedence over the provider's start command
        if let Some(procfile_start) = self.get_procfile_start_cmd(app)? {
            let mut start_phase = plan.start_phase.clone().unwrap_or_default();
            start_phase.cmd = Some(procfile_start);
            plan.set_start_phase(start_phase);
        }

        // The Procfiles release command is append to the provider's build command
        if let Some(procfile_release) = self.get_procfile_release_cmd(app)? {
            if let Some(build) = plan.get_phase_mut("build") {
                build.add_cmd(procfile_release);
            }
        }

        plan = BuildPlan::apply_config(&plan, &config);

        if !environment.get_variable_names().is_empty() {
            plan.add_variables(Environment::clone_variables(environment));
        }

        Ok(plan)
    }

    fn get_procfile_start_cmd(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let mut procfile: HashMap<String, String> =
                app.read_yaml("Procfile").context("Reading Procfile")?;
            procfile.remove("release");
            if procfile.is_empty() {
                Ok(None)
            } else {
                let process = procfile.values().collect::<Vec<_>>()[0].to_string();
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
