use super::{npm::PackageJson, Provider};
use crate::bb::app::App;
use anyhow::Result;

pub struct YarnProvider {}

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("package.json") && app.includes_file("yarn.lock"))
    }

    fn pkgs(&self, _app: &App) -> String {
        "pkgs.stdenv pkgs.yarn".to_string()
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if package_json.scripts.get("build").is_some() {
            return Ok(Some("yarn build".to_string()));
        }

        Ok(None)
    }

    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn start".to_string()))
    }
}
