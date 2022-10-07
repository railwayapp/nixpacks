use std::path::Path;

use crate::{
    nixpacks::{
        app::App,
        environment::{Environment, EnvironmentVariables},
        plan::{BuildPlan, PlanGenerator},
    },
    providers::{procfile::ProcfileProvider, Provider},
};
use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;

use super::merge::Mergeable;

const NIXPACKS_METADATA: &str = "NIXPACKS_METADATA";

#[derive(Clone, Default, Debug)]
pub struct GeneratePlanOptions {
    pub plan: Option<BuildPlan>,
    pub config_file: Option<String>,
}

pub struct NixpacksBuildPlanGenerator<'a> {
    providers: &'a [&'a dyn Provider],
    config: GeneratePlanOptions,
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
        config: GeneratePlanOptions,
    ) -> NixpacksBuildPlanGenerator<'a> {
        NixpacksBuildPlanGenerator { providers, config }
    }

    /// Get a build plan from the provider and by applying the config from the environment
    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<BuildPlan> {
        let file_plan = self.read_file_plan(app, env)?;
        let env_plan = BuildPlan::from_environment(env);
        let cli_plan = self.config.plan.clone().unwrap_or_default();
        let plan_before_providers = BuildPlan::merge_plans(&vec![file_plan, env_plan, cli_plan]);

        let provider_plan =
            self.get_plan_from_providers(plan_before_providers.providers.clone(), app, env)?;

        let procfile_plan = (ProcfileProvider {})
            .get_build_plan(app, env)?
            .unwrap_or_default();

        let mut plan =
            BuildPlan::merge_plans(&vec![provider_plan, procfile_plan, plan_before_providers]);

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

    fn read_file_plan(&self, app: &App, env: &Environment) -> Result<BuildPlan> {
        let file_path = if let Some(file_path) = &self.config.config_file {
            Some(file_path.clone())
        } else if let Some(env_config_file) = env.get_config_variable("CONFIG_FILE") {
            if !app.includes_file(&env_config_file) {
                bail!("Config file {} does not exist", env_config_file);
            }

            Some(env_config_file)
        } else if app.includes_file("nixpacks.toml") {
            Some("nixpacks.toml".to_owned())
        } else if app.includes_file("nixpacks.json") {
            Some("nixpacks.json".to_owned())
        } else {
            None
        };

        let plan =
            if let Some(file_path) = file_path {
                let filename = Path::new(&file_path);
                let ext = filename.extension().unwrap_or_default();

                let contents = app.read_file(file_path.as_str()).with_context(|| {
                    format!("Failed to read Nixpacks config file `{}`", file_path)
                })?;
                let plan = if ext == "toml" {
                    BuildPlan::from_toml(&contents)
                } else if ext == "json" {
                    BuildPlan::from_json(&contents)
                } else {
                    bail!("Unknown file type: {}", file_path)
                };

                Some(plan.with_context(|| {
                    format!("Failed to parse Nixpacks config file `{}`", file_path)
                })?)
            } else {
                None
            };

        if plan.is_some() {
            println!(
                "{}",
                "\n Nixpacks file based configuration is experimental and may change\n"
                    .bright_yellow()
            );
        }

        Ok(plan.unwrap_or_default())
    }
}
