use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
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

    fn start(&self, _app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        let mut start_phase = StartPhase::new(format!("./{}", BINARY_NAME));

        let cgo = env
            .get_variable("CGO_ENABLED")
            .cloned()
            .unwrap_or_else(|| "0".to_string());

        // Only run in a new image if CGO_ENABLED=0 (default)
        if cgo != "1" {
            start_phase.run_in_slim_image();
        }

        Ok(Some(start_phase))
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(EnvironmentVariables::from([(
            "CGO_ENABLED".to_string(),
            "0".to_string(),
        )])))
    }
}
