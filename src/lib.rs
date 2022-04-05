use std::{env, fs};

use crate::{
    nixpacks::{
        app::App, logger::Logger, plan::BuildPlan, AppBuilder, AppBuilderOptions,
        EnvironmentVariable,
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
    variables: Vec<EnvironmentVariable>,
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
    let mut app_builder = AppBuilder::new(None, &app, &logger, variables, &options)?;

    let plan = app_builder.plan(providers)?;
    Ok(plan)
}

pub fn build(
    path: &str,
    name: Option<String>,
    custom_pkgs: Vec<&str>,
    custom_build_cmd: Option<String>,
    custom_start_cmd: Option<String>,
    pin_pkgs: bool,
    variables: Vec<EnvironmentVariable>,
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
    let mut app_builder = AppBuilder::new(name, &app, &logger, variables, &options)?;

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

pub fn get_environment_variables(envs: Vec<&str>) -> Result<Vec<EnvironmentVariable>> {
    let mut variables: Vec<EnvironmentVariable> = Vec::new();
    for env in envs {
        let v: Vec<&str> = env.split("=").collect();
        if v.len() == 1 {
            // Pull the variable from the current environment
            let name = v[0];
            if let Ok(value) = env::var(name) {
                // Variable is set
                variables.push(EnvironmentVariable(name.to_string(), value));
            }
        } else if v.len() > 2 {
            bail!("Unable to parse variable string");
        } else {
            // Use provided name, value pair
            variables.push(EnvironmentVariable(v[0].to_string(), v[1].to_string()));
        }
    }

    return Ok(variables);
}

#[cfg(test)]
mod tests {
    use crate::{get_environment_variables, nixpacks::EnvironmentVariable};

    #[test]
    fn test_environment_variable_parsing() {
        let variables =
            get_environment_variables(vec!["HELLO=world", "CARGO_PKG_NAME", "NON_EXISTANT"])
                .unwrap();
        assert_eq!(
            variables,
            vec![
                EnvironmentVariable("HELLO".to_string(), "world".to_string()),
                EnvironmentVariable("CARGO_PKG_NAME".to_string(), "nixpacks".to_string())
            ]
        );

        assert!(get_environment_variables(vec!["INVALID=ENV=CONFIG"]).is_err());
    }
}
