use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::{NixConfig, Pkg},
    phase::{InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub struct GolangProvider {}

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "golang"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.go"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<SetupPhase> {
        Ok(SetupPhase::new(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.go"),
        ])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<InstallPhase> {
        if app.includes_file("go.mod") {
            return Ok(InstallPhase::new("go get".to_string()));
        }
        Ok(InstallPhase::default())
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<StartPhase> {
        Ok(StartPhase::new("go run main.go".to_string()))
    }
}
