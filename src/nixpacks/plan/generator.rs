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

use super::{
    merge::Mergeable,
    utils::{fill_auto_in_vec, remove_autos_from_vec},
};

const NIXPACKS_METADATA: &str = "NIXPACKS_METADATA";

#[derive(Clone, Default, Debug)]
pub struct GeneratePlanOptions {
    pub plan: Option<BuildPlan>,
    pub config_file: Option<String>,
}

pub struct NixpacksBuildPlanGenerator<'a> {
    providers: &'a [&'a (dyn Provider)],
    config: GeneratePlanOptions,
}

impl<'a> PlanGenerator for NixpacksBuildPlanGenerator<'a> {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<(BuildPlan, App)> {
        // If the provider defines a build plan in the new format, use that
        let plan = self.get_build_plan(app, environment)?;

        Ok(plan)
    }

    fn get_plan_providers(&self, app: &App, env: &Environment) -> Result<Vec<String>> {
        let plan_before_providers = self.get_plan_before_providers(app, env)?;
        let providers = self.get_all_providers(app, env, plan_before_providers.providers)?;

        Ok(providers)
    }
}

impl NixpacksBuildPlanGenerator<'_> {
    pub fn new<'a>(
        providers: &'a [&'a (dyn Provider)],
        config: GeneratePlanOptions,
    ) -> NixpacksBuildPlanGenerator<'a> {
        NixpacksBuildPlanGenerator { providers, config }
    }

    /// Get a build plan from the provider and by applying the config from the environment
    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<(BuildPlan, App)> {
        let plan_before_providers = self.get_plan_before_providers(app, env)?;

        // Add the variables from the nixpacks.toml to environment
        let new_env = &Environment::append_variables(
            env,
            plan_before_providers.variables.clone().unwrap_or_default(),
        );

        let provider_plan =
            self.get_plan_from_providers(app, new_env, plan_before_providers.providers.clone())?;

        let procfile_plan = (ProcfileProvider {})
            .get_build_plan(app, new_env)?
            .unwrap_or_default();

        let mut plan =
            BuildPlan::merge_plans(&vec![provider_plan, procfile_plan, plan_before_providers]);

        if !new_env.get_variable_names().is_empty() {
            plan.add_variables(Environment::clone_variables(new_env));
        }

        plan.pin(new_env.is_config_variable_truthy("DEBIAN"));
        if plan.clone().phases.unwrap_or_default().is_empty() {
            // try again in a subdir
            let dir_count = app.paths.clone().iter().filter(|p| p.is_dir()).count();
            if dir_count == 1 {
                // there is 1 sub dir, try and generate a plan from that
                let paths = app.paths.clone();
                let new_dir = paths.iter().find(|p| p.is_dir()).unwrap();
                return self
                    .get_build_plan(&App::new(new_dir.display().to_string().as_str())?, env);
            }
        }
        Ok((plan, app.clone()))
    }

    fn get_plan_before_providers(&self, app: &App, env: &Environment) -> Result<BuildPlan> {
        let file_plan = self.read_file_plan(app, env)?;
        let env_plan = BuildPlan::from_environment(env);
        let cli_plan = self.config.plan.clone().unwrap_or_default();
        let plan_before_providers = BuildPlan::merge_plans(&vec![file_plan, env_plan, cli_plan]);

        Ok(plan_before_providers)
    }

    fn get_detected_providers(&self, app: &App, env: &Environment) -> Result<Vec<String>> {
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

    /// Get all the providers that will be used to create the plan
    pub fn get_all_providers(
        &self,
        app: &App,
        env: &Environment,
        manually_providers: Option<Vec<String>>,
    ) -> Result<Vec<String>> {
        let detected_providers = self.get_detected_providers(app, env)?;
        let provider_names = remove_autos_from_vec(
            fill_auto_in_vec(
                Some(detected_providers),
                Some(manually_providers.unwrap_or_else(|| vec!["...".to_string()])),
            )
            .unwrap_or_default(),
        );

        Ok(provider_names)
    }

    fn get_plan_from_providers(
        &self,
        app: &App,
        env: &Environment,
        manual_providers: Option<Vec<String>>,
    ) -> Result<BuildPlan> {
        let provider_names = self.get_all_providers(app, env, manual_providers)?;

        if provider_names.len() > 1 {
            println!(
                "{}",
                "\n Using multiple providers is experimental\n".bright_yellow()
            );
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

                    plan = BuildPlan::merge(&provider_plan, &plan);
                }
            } else if name != "..." && name != "@auto" {
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
                    format!("Failed to read Nixpacks config file `{file_path}`")
                })?;
                let plan = if ext == "toml" {
                    BuildPlan::from_toml(&contents)
                } else if ext == "json" {
                    BuildPlan::from_json(&contents)
                } else {
                    bail!("Unknown file type: {}", file_path)
                };

                Some(plan.with_context(|| {
                    format!("Failed to parse Nixpacks config file `{file_path}`")
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
