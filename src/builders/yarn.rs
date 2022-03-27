use anyhow::Result;
use std::path::PathBuf;

use super::Builder;

pub struct YarnBuilder {}

impl Builder for YarnBuilder {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, paths: Vec<PathBuf>) -> Result<bool> {
        for path in paths {
            if path.file_name().unwrap() == "yarn.lock" {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn build_inputs(&self) -> String {
        "pkgs.stdenv pkgs.yarn".to_string()
    }

    fn install_cmd(&self) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self) -> Result<Option<String>> {
        Ok(Some("yarn build".to_string()))
    }

    fn suggested_start_command(&self) -> Result<Option<String>> {
        Ok(Some("yarn start".to_string()))
    }
}
