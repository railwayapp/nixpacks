use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub mod crystal;
pub mod deno;
pub mod go;
pub mod haskell;
pub mod node;
pub mod python;
pub mod ruby;
pub mod rust;

pub trait Provider {
    fn name(&self) -> &str;
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;
    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(None)
    }
    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        Ok(None)
    }
    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(None)
    }
    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(None)
    }
    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(None)
    }
}
