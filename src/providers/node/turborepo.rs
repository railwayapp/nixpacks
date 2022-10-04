use serde_json::Value;
use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::nixpacks::{app::App, environment::Environment};

use super::PackageJson;

#[derive(Debug, Deserialize, Serialize)]
pub struct TurboJson {
    pub pipeline: HashMap<String, Value>,
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
        if cfg.pipeline.contains_key(name) {
            Some(format!("npx turbo run {}", name))
        } else {
            None
        }
    }

    pub fn get_build_cmd(cfg: &TurboJson) -> Option<String> {
        Turborepo::get_pipeline_cmd(cfg, "build")
    }

    pub fn get_start_cmd(cfg: &TurboJson) -> Option<String> {
        Turborepo::get_pipeline_cmd(cfg, "start")
    }

    pub fn get_app_name(env: &Environment) -> Option<String> {
        env.get_config_variable("TURBO_APP_NAME")
    }

    pub fn has_app(app: &App, workspaces: Vec<String>, name: String) -> Result<bool> {
        //TODO: parallelize?
        for glob in workspaces {
            let files = app.find_directories(&glob)?;
            for file in files {
                if file.ends_with(format!("/{}", name)) {
                    return Ok(true)
                }
            }
        }
        Ok(false)
    }
}
