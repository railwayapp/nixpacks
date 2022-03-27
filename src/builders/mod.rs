use anyhow::Result;
use std::path::PathBuf;

pub trait Builder {
    fn name(&self) -> &str;
    fn detect(&self, paths: Vec<PathBuf>) -> Result<bool>;
    fn build_inputs(&self) -> String;
    fn install_cmd(&self) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self) -> Result<Option<String>> {
        Ok(None)
    }
}

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
