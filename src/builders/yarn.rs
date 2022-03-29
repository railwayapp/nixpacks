use super::Builder;
use crate::bb::app::App;
use anyhow::Result;

pub struct YarnBuilder {}

impl Builder for YarnBuilder {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("package.json") && app.includes_file("yarn.lock"))
    }

    fn build_inputs(&self, _app: &App) -> String {
        "pkgs.stdenv pkgs.yarn".to_string()
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn build".to_string()))
    }

    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("yarn start".to_string()))
    }
}
