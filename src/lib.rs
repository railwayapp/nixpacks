use std::{env, fs};

use crate::{
    nixpacks::{
        app::App, environment::Environment, logger::Logger, plan::BuildPlan, AppBuilder,
        AppBuilderOptions,
    },
    providers::{
        deno::DenoProvider, go::GolangProvider, npm::NpmProvider, rust::RustProvider,
        yarn::YarnProvider, Pkg,
    },
};
use anyhow::{bail, Context, Result};
use providers::Provider;

pub mod nixpacks;
pub mod providers;

pub fn get_providers() -> Vec<&'static dyn Provider> {
    vec![
        &YarnProvider {},
        &NpmProvider {},
        &GolangProvider {},
        &RustProvider {},
        &DenoProvider {},
    ]
}

pub fn gen_plan(
    path: &str,
    custom_pkgs: Vec<&str>,
    custom_build_cmd: Option<String>,
    custom_start_cmd: Option<String>,
    envs: Vec<&str>,
    pin_pkgs: bool,
) -> Result<BuildPlan> {
    let logger = Logger::new();
    let providers = get_providers();

    let options = AppBuilderOptions {
        custom_pkgs: custom_pkgs.iter().map(|p| Pkg::new(p)).collect(),
        custom_build_cmd,
        custom_start_cmd,
        pin_pkgs,
    };

    let app = App::new(path)?;
    let environment = create_environment(envs)?;
    let mut app_builder = AppBuilder::new(None, &app, &environment, &logger, &options)?;

    let plan = app_builder.plan(providers)?;
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    path: &str,
    name: Option<String>,
    custom_pkgs: Vec<&str>,
    custom_build_cmd: Option<String>,
    custom_start_cmd: Option<String>,
    pin_pkgs: bool,
    envs: Vec<&str>,
    plan_path: Option<&str>,
) -> Result<()> {
    let logger = Logger::new();
    let providers = get_providers();

    let options = AppBuilderOptions {
        custom_pkgs: custom_pkgs.iter().map(|p| Pkg::new(p)).collect(),
        custom_build_cmd,
        custom_start_cmd,
        pin_pkgs,
    };

    let app = App::new(path)?;
    let environment = create_environment(envs)?;
    let mut app_builder = AppBuilder::new(name, &app, &environment, &logger, &options)?;

    match plan_path {
        Some(plan_path) => {
            let plan_json = fs::read_to_string(plan_path).context("Reading build plan")?;
            let plan: BuildPlan =
                serde_json::from_str(&plan_json).context("Deserializing build plan")?;
            app_builder.build_from_plan(&plan)?;
        }
        None => {
            app_builder.build(providers)?;
        }
    }

    Ok(())
}

pub fn create_environment(envs: Vec<&str>) -> Result<Environment> {
    let mut environment = Environment::default();
    for env in envs {
        let v: Vec<&str> = env.split('=').collect();
        if v.len() == 1 {
            // Pull the variable from the current environment
            let name = v[0];
            if let Ok(value) = env::var(name) {
                // Variable is set
                environment.set_variable(name.to_string(), value);
            }
        } else if v.len() > 2 {
            bail!("Unable to parse variable string");
        } else {
            // Use provided name, value pair
            environment.set_variable(v[0].to_string(), v[1].to_string());
        }
    }

    Ok(environment)
}

#[cfg(test)]
mod tests {
    use crate::create_environment;

    #[test]
    fn test_environment_variable_parsing() {
        let environment =
            create_environment(vec!["HELLO=world", "CARGO_PKG_NAME", "NON_EXISTANT"]).unwrap();
        assert_eq!(
            environment.get_variable("HELLO"),
            Some(&"world".to_string())
        );
        assert_eq!(
            environment.get_variable("CARGO_PKG_NAME"),
            Some(&"nixpacks".to_string())
        );
        assert!(environment.get_variable("NON_EXISTANT").is_none());
    }

    #[test]
    fn test_create_invalid_environment() {
        assert!(create_environment(vec!["INVALID=ENV=CONFIG"]).is_err());
    }
}
