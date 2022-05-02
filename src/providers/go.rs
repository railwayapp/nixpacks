use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub struct GolangProvider {}

pub const BINARY_NAME: &'static &str = &"out";

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "golang"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.go") || app.includes_file("go.mod"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("go")])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        if app.includes_file("go.mod") {
            return Ok(Some(InstallPhase::new("go get".to_string())));
        }
        Ok(None)
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        if app.includes_file("go.mod") {
            Ok(Some(BuildPhase::new(format!(
                "go build -o {}",
                BINARY_NAME
            ))))
        } else {
            Ok(Some(BuildPhase::new(format!(
                "go build -o {} main.go",
                BINARY_NAME
            ))))
        }
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(format!("./{}", BINARY_NAME))))
    }
}
