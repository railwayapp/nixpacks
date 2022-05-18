use std::env;

use crate::{
    nixpacks::{
        app::App, environment::Environment, logger::Logger, nix::Pkg, plan::BuildPlan, AppBuilder,
        AppBuilderOptions,
    },
    providers::{
        crystal::CrystalProvider, deno::DenoProvider, go::GolangProvider,
        haskell::HaskellStackProvider, node::NodeProvider, python::PythonProvider,
        rust::RustProvider,
    },
};
use anyhow::{bail, Result};
use providers::Provider;

pub(crate) mod chain;
pub mod nixpacks;
pub mod providers;

pub fn get_providers() -> Vec<&'static dyn Provider> {
    vec![
        &GolangProvider {},
        &DenoProvider {},
        &NodeProvider {},
        &RustProvider {},
        &PythonProvider {},
        &HaskellStackProvider {},
        &CrystalProvider {},
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
        out_dir: None,
        plan_path: None,
        tags: Vec::new(),
        quiet: false,
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
    plan_path: Option<String>,
    out_dir: Option<String>,
    tags: Vec<&str>,
    quiet: bool,
) -> Result<()> {
    let logger = Logger::new();
    let providers = get_providers();

    let options = AppBuilderOptions {
        custom_pkgs: custom_pkgs.iter().map(|p| Pkg::new(p)).collect(),
        custom_build_cmd,
        custom_start_cmd,
        pin_pkgs,
        out_dir,
        plan_path,
        tags: tags.iter().map(|s| s.to_string()).collect(),
        quiet,
    };

    let app = App::new(path)?;
    let environment = create_environment(envs)?;
    let mut app_builder = AppBuilder::new(name, &app, &environment, &logger, &options)?;

    app_builder.build(providers)?;

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
