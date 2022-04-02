use super::{npm::PackageJson, Pkg, Provider};
use crate::nixpacks::app::App;
use anyhow::Result;

pub struct YarnProvider {}

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("package.json") && app.includes_file("yarn.lock"))
    }

    fn pkgs(&self, _app: &App) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.yarn")]
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("build").is_some() {
                return Ok(Some("yarn build".to_string()));
            }
        }

        Ok(None)
    }

    fn suggested_start_command(&self, app: &App) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("start").is_some() {
                return Ok(Some("yarn start".to_string()));
            }
        }

        if app.includes_file("index.js") {
            return Ok(Some("node index.js".to_string()));
        }

        Ok(None)
    }
}
