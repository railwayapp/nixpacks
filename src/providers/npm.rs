use std::collections::HashMap;

use super::{Pkg, Provider};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct NpmProvider {}

impl Provider for NpmProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn pkgs(&self, _app: &App, _env: &Environment) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.nodejs")]
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("npm ci".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("build").is_some() {
                return Ok(Some("npm run build".to_string()));
            }
        }

        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("start").is_some() {
                return Ok(Some("npm run start".to_string()));
            }
        }

        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                return Ok(Some(format!("node {}", main)));
            }
        }
        if app.includes_file("index.js") {
            return Ok(Some(String::from("node index.js")));
        }

        Ok(None)
    }

    fn get_environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        let mut variables = EnvironmentVariables::default();
        variables.insert("NODE_ENV".to_string(), "production".to_string());
        variables.insert("NPM_CONFIG_PRODUCTION".to_string(), "false".to_string());

        Ok(variables)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub scripts: Option<HashMap<String, String>>,
    pub main: Option<String>,
}
