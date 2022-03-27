use anyhow::Result;
use std::path::PathBuf;

use super::Builder;

pub struct NpmBuilder {}

impl Builder for NpmBuilder {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, paths: Vec<PathBuf>) -> Result<bool> {
        for path in paths {
            if path.file_name().unwrap() == "package.json" {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn build_inputs(&self) -> String {
        "pkgs.stdenv pkgs.nodejs".to_string()
    }

    fn install_cmd(&self) -> Result<Option<String>> {
        Ok(Some("npm install".to_string()))
    }

    fn suggested_build_cmd(&self) -> Result<Option<String>> {
        Ok(Some("npm run build".to_string()))
    }

    fn suggested_start_command(&self) -> Result<Option<String>> {
        Ok(Some("npm run start".to_string()))
    }
}
