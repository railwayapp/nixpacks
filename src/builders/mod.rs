use crate::bb::app::App;
use anyhow::Result;

pub mod npm;
pub mod yarn;

pub trait Builder {
    fn name(&self) -> &str;
    fn detect(&self, app: &App) -> Result<bool>;
    fn build_inputs(&self, app: &App) -> String;
    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
}
