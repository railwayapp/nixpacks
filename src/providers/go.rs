use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::{Pkg},
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

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.go"),
        ])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        if app.includes_file("go.mod") {
            return Ok(Some(InstallPhase::new("go get".to_string())));
        }
        Ok(None)
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new("go run main.go".to_string())))
    }
}
