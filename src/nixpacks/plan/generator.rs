use crate::{
    nixpacks::{
        app::App,
        environment::{Environment, EnvironmentVariables},
        nix::pkg::Pkg,
        plan::{BuildPlan, PlanGenerator},
    },
    providers::{procfile::ProcfileProvider, Provider},
};
use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;

use super::merge::Mergeable;

// This line is automatically updated.
// Last Modified: 2022-08-29 17:07:50 UTC+0000
// https://github.com/NixOS/nixpkgs/commit/0e304ff0d9db453a4b230e9386418fd974d5804a
pub const NIXPKGS_ARCHIVE: &str = "0e304ff0d9db453a4b230e9386418fd974d5804a";
const NIXPACKS_METADATA: &str = "NIXPACKS_METADATA";

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
    config: BuildPlan,
}

impl<'a> PlanGenerator for NixpacksBuildPlanGenerator<'a> {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan> {
        // If the provider defines a build plan in the new format, use that
        let plan = self.get_build_plan(app, environment)?;

        Ok(plan)
    }
}

impl NixpacksBuildPlanGenerator<'_> {
    pub fn new<'a>(
        providers: &'a [&'a dyn Provider],
        config: BuildPlan,
    ) -> NixpacksBuildPlanGenerator<'a> {
        NixpacksBuildPlanGenerator { providers, config }
    }

    /// Get a build plan from the provider and by applying the config from the environment
    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<BuildPlan> {
        let file_plan = self.read_file_plan(app)?;
        let env_plan = BuildPlan::from_environment(env);
        let cli_plan = self.config.clone();
        let plan_before_providers = BuildPlan::merge_plans(&vec![file_plan, env_plan, cli_plan]);

        let provider_plan =
            self.get_plan_from_providers(plan_before_providers.providers.clone(), app, env)?;

        let procfile_plan = (ProcfileProvider {})
            .get_build_plan(app, env)?
            .unwrap_or_default();

        let mut plan =
            BuildPlan::merge_plans(&vec![provider_plan, plan_before_providers, procfile_plan]);

        if !env.get_variable_names().is_empty() {
            plan.add_variables(Environment::clone_variables(env));
        }

        plan.pin();

        Ok(plan)
    }

    fn get_auto_providers(&self, app: &App, env: &Environment) -> Result<Vec<String>> {
        let mut providers = Vec::new();

        for provider in self.providers {
            if provider.detect(app, env)? {
                providers.push(provider.name().to_string());

                // Only match a single provider... for now
                break;
            }
        }

        Ok(providers)
    }

    fn get_plan_from_providers(
        &self,
        provider_names: Option<Vec<String>>,
        app: &App,
        env: &Environment,
    ) -> Result<BuildPlan> {
        let provider_names = if let Some(provider_names) = provider_names {
            provider_names
        } else {
            self.get_auto_providers(app, env)?
        };

        if provider_names.len() > 1 {
            bail!("Only a single provider is supported at this time");
        }

        let mut plan = BuildPlan::default();
        let mut count = 0;

        let mut metadata = Vec::new();

        for name in provider_names {
            let provider = self.providers.iter().find(|p| p.name() == name);
            if let Some(provider) = provider {
                if let Some(mut provider_plan) = provider.get_build_plan(app, env)? {
                    // All but the first provider have their phases prefixed with their name
                    if count > 0 {
                        provider_plan.prefix_phases(provider.name());
                    }

                    let metadata_string = provider
                        .metadata(app, env)?
                        .join_as_comma_separated(provider.name().to_owned());
                    metadata.push(metadata_string);

                    plan = BuildPlan::merge(&plan, &provider_plan);
                }
            } else {
                bail!("Provider {} not found", name);
            }

            count += 1;
        }

        if count > 0 {
            plan.add_variables(EnvironmentVariables::from([(
                NIXPACKS_METADATA.to_string(),
                metadata.join(","),
            )]));
        }

        Ok(plan)
    }

    fn read_file_plan(&self, app: &App) -> Result<BuildPlan> {
        let build_plan = if app.includes_file("nixpacks.json") {
            let contents = app.read_file("nixpacks.json")?;
            let mut config: BuildPlan = serde_json::from_str(contents.as_str())
                .context("failed to parse config from nixpacks.json")?;
            config.resolve_phase_names();
            Some(config)
        } else if app.includes_file("nixpacks.toml") {
            let contents = app.read_file("nixpacks.toml")?;
            let mut config: BuildPlan = toml::from_str(contents.as_str())
                .context("failed to parse config from nixpacks.toml")?;
            config.resolve_phase_names();
            Some(config)
        } else {
            None
        };

        if build_plan.is_some() {
            println!(
                "{}",
                "\nNixpacks file based configuration is experimental and may change\n"
                    .bright_yellow()
            );
        }

        Ok(build_plan.unwrap_or_default())
    }
}
