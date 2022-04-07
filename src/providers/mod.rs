use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    pkg::Pkg,
};
use anyhow::Result;

pub mod deno;
pub mod go;
pub mod npm;
pub mod rust;
pub mod yarn;

pub trait Provider {
    fn name(&self) -> &str;
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;
    fn pkgs(&self, app: &App, _env: &Environment) -> Result<Vec<Pkg>>;
    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn get_environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        Ok(EnvironmentVariables::default())
    }
}
