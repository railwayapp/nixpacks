use super::Provider;
use crate::{
    nixpacks::{app::App, environment::Environment},
    providers::Pkg,
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

    fn pkgs(&self, _app: &App, _env: &Environment) -> Result<Vec<Pkg>> {
        Ok(vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.go")])
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        if _app.includes_file("go.mod") {
            return Ok(Some("go get".to_string()));
        }
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("go run main.go".to_string()))
    }
}
