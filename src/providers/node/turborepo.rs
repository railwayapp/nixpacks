use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::nixpacks::{app::App, environment::Environment};

#[derive(Debug, Deserialize, Serialize)]
pub struct TurboJson {
    pub pipeline: HashMap<String, Pipeline>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    // Fields not used
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
}
