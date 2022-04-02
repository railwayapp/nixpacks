use std::fs;

use crate::{
    nixpacks::{app::App, logger::Logger, plan::BuildPlan, AppBuilder, AppBuilderOptions},
    providers::{
        go::GolangProvider, npm::NpmProvider, rust::RustProvider, yarn::YarnProvider, Pkg,
    },
};
use anyhow::{Context, Result};
use providers::Provider;

pub mod nixpacks;
pub mod providers;

pub fn get_providers() -> Vec<&'static dyn Provider> {
    vec![
        &YarnProvider {},
        &NpmProvider {},
        &GolangProvider {},
        &RustProvider {},
    ]
}

pub fn gen_plan(
    path: &str,
    custom_pkgs: Vec<&str>,
    custom_build_cmd: Option<String>,
    custom_start_cmd: Option<String>,
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
    let mut app_builder = AppBuilder::new(None, &app, &logger, &options)?;

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
    let mut app_builder = AppBuilder::new(name, &app, &logger, &options)?;

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
