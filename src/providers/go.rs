use super::Provider;
use crate::bb::app::App;
use anyhow::Result;

pub struct GolangProvider {}

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "golang"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("main.go"))
    }

    fn pkgs(&self, _app: &App) -> String {
        "pkgs.stdenv pkgs.go".to_string()
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        if _app.includes_file("go.mod") {
            return Ok(Some("go get".to_string()));
        }
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("go run main.go".to_string()))
    }
}
