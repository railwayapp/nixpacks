use anyhow::Result;
use std::path::PathBuf;

use crate::bb::AppSource;

use super::Builder;

pub struct YarnBuilder {}

impl Builder for YarnBuilder {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, app: &AppSource) -> Result<bool> {
        Ok(app.includes_file("package.json") && app.includes_file("yarn.lock"))
    }

    fn build_inputs(&self, _app: &AppSource) -> String {
        "pkgs.stdenv pkgs.yarn".to_string()
    }

    fn install_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("yarn build".to_string()))
    }

    fn suggested_start_command(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("yarn start".to_string()))
    }
}
