use serde_json::Value;
use std::{collections::HashMap, error::Error};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    nixpacks::{app::App, environment::Environment},
    providers::node::Workspaces,
};

use super::{NodeProvider, PackageJson};

#[derive(Debug, Deserialize, Serialize)]
pub struct TurboJson {
    #[serde(default)]
    pub pipeline: HashMap<String, Value>,
    #[serde(default)]
    pub tasks: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PnpmWorkspaces {
    packages: Vec<String>,
}

pub fn pnpm_workspaces(app: &App) -> Result<Vec<String>> {
    let workspaces: PnpmWorkspaces = app.read_yaml("pnpm-workspace.yaml")?;

    Ok(workspaces.packages)
}

pub struct Turborepo;

impl Turborepo {
    pub fn is_turborepo(app: &App) -> bool {
        app.includes_file("turbo.json")
    }

    pub fn get_config(app: &App) -> Result<TurboJson> {
        app.read_json("turbo.json")
    }

    fn get_pipeline_cmd(cfg: &TurboJson, name: &str) -> Option<String> {
        if cfg.pipeline.contains_key(name) || cfg.tasks.contains_key(name) {
            Some(format!("npx turbo run {name}"))
        } else {
            None
        }
    }

    pub fn get_build_cmd(cfg: &TurboJson) -> Option<String> {
        Turborepo::get_pipeline_cmd(cfg, "build")
    }

    pub fn get_actual_build_cmd(
        app: &App,
        env: &Environment,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let turbo_cfg = Turborepo::get_config(app)?;
        let dlx = NodeProvider::get_package_manager_dlx_command(app);
        if let Some(build_cmd) = Turborepo::get_build_cmd(&turbo_cfg) {
            return Ok(Some(build_cmd));
        } else if let Some(app_name) = Turborepo::get_app_name(env) {
            return Ok(Some(format!("{dlx} turbo run {app_name}:build")));
        }

        Ok(None)
    }

    pub fn get_start_cmd(cfg: &TurboJson) -> Option<String> {
        Turborepo::get_pipeline_cmd(cfg, "start")
    }

    pub fn get_actual_start_cmd(
        app: &App,
        env: &Environment,
        package_json: &PackageJson,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let turbo_cfg = Turborepo::get_config(app)?;
        let app_name = Turborepo::get_app_name(env);
        let pkg_manager = NodeProvider::get_package_manager(app);

        if let Some(name) = app_name {
            if Turborepo::has_app(
                app,
                if pkg_manager == "pnpm" {
                    pnpm_workspaces(app)?
                } else if let Some(Workspaces::Array(workspaces)) = &package_json.workspaces {
                    workspaces.clone()
                } else {
                    Vec::default()
                },
                &name,
            )? {
                return Ok(Some(if pkg_manager == "pnpm" {
                    format!("pnpm --filter {name} run start")
                } else if pkg_manager == "yarn" {
                    format!("{pkg_manager} workspace {name} run start")
                } else {
                    format!("{pkg_manager} --workspace {name} run start")
                }));
            }
            eprintln!("Warning: Turborepo app `{name}` not found");
        }
        if let Some(start_pipeline) = Turborepo::get_start_cmd(&turbo_cfg) {
            return Ok(Some(start_pipeline));
        }
        Ok(None)
    }

    pub fn get_app_name(env: &Environment) -> Option<String> {
        env.get_config_variable("TURBO_APP_NAME")
    }

    pub fn has_app(app: &App, workspaces: Vec<String>, name: &str) -> Result<bool> {
        for glob in workspaces {
            let files = app.find_directories(&glob)?;
            for file in files {
                if file.ends_with(name) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
