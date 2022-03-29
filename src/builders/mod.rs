use anyhow::Result;

use crate::bb::AppSource;

pub mod npm;
pub mod yarn;

pub trait Builder {
    fn name(&self) -> &str;
    fn detect(&self, app: &AppSource) -> Result<bool>;
    fn build_inputs(&self, app: &AppSource) -> String;
    fn install_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, _app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
}
