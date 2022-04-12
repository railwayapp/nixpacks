use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
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
    fn setup(&self, _app: &App, _env: &Environment) -> Result<SetupPhase> {
        Ok(SetupPhase::default())
    }
    fn install(&self, _app: &App, _env: &Environment) -> Result<InstallPhase> {
        Ok(InstallPhase::default())
    }
    fn build(&self, _app: &App, _env: &Environment) -> Result<BuildPhase> {
        Ok(BuildPhase::default())
    }
    fn start(&self, _app: &App, _env: &Environment) -> Result<StartPhase> {
        Ok(StartPhase::default())
    }
    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        Ok(EnvironmentVariables::default())
    }
}
