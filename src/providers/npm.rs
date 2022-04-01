use std::collections::HashMap;

use super::{Pkg, Provider};
use crate::nixpacks::app::App;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct NpmProvider {}

impl Provider for NpmProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn pkgs(&self, _app: &App) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.nodejs")]
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("npm install".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if package_json.scripts.get("build").is_some() {
            return Ok(Some("npm run build".to_string()));
        }

        Ok(None)
    }

    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("npm run start".to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub scripts: HashMap<String, String>,
}
