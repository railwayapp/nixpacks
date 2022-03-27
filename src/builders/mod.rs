use anyhow::Result;
use std::path::PathBuf;

pub trait Builder {
    fn name(&self) -> &str;
    fn detect(&self, paths: Vec<PathBuf>) -> Result<bool>;
    fn build_inputs(&self) -> Vec<String>;
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

pub struct StdEnvBuilder {}

impl Builder for StdEnvBuilder {
    fn name(&self) -> &str {
        "stdenv"
    }

    fn detect(&self, _paths: Vec<PathBuf>) -> Result<bool> {
        Ok(true)
    }

    fn build_inputs(&self) -> Vec<String> {
        vec!["pkgs.stdenv".to_string()]
    }
}

pub struct NodeBuilder {}

impl Builder for NodeBuilder {
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

    fn build_inputs(&self) -> Vec<String> {
        vec!["pkgs.nodejs".to_string()]
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
