use anyhow::Result;
use std::path::PathBuf;

pub mod npm;
pub mod yarn;

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
