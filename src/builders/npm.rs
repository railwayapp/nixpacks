use anyhow::Result;

use crate::bb::AppSource;

use super::Builder;

pub struct NpmBuilder {}

impl Builder for NpmBuilder {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &AppSource) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn build_inputs(&self, _app: &AppSource) -> String {
        "pkgs.stdenv pkgs.nodejs".to_string()
    }

    fn install_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("npm install".to_string()))
    }

    fn suggested_build_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("npm run build".to_string()))
    }

    fn suggested_start_command(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(Some("npm run start".to_string()))
    }
}
