use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    plan::{
        legacy_phase::{LegacyBuildPhase, LegacyInstallPhase, LegacySetupPhase, LegacyStartPhase},
        BuildPlan,
    },
};
use anyhow::Result;

pub mod clojure;
pub mod crystal;
pub mod csharp;
pub mod dart;
pub mod deno;
pub mod fsharp;
pub mod go;
pub mod haskell;
pub mod java;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod staticfile;
pub mod swift;
pub mod zig;

pub trait Provider {
    fn name(&self) -> &str;

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;

    fn get_build_plan(&self, _app: &App, _environment: &Environment) -> Result<Option<BuildPlan>> {
        Ok(None)
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<LegacySetupPhase>> {
        Ok(None)
    }
    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<LegacyInstallPhase>> {
        Ok(None)
    }
    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<LegacyBuildPhase>> {
        Ok(None)
    }
    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<LegacyStartPhase>> {
        Ok(None)
    }
    fn static_assets(&self, _app: &App, _env: &Environment) -> Result<Option<StaticAssets>> {
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
