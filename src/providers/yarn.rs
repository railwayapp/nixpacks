use super::{npm::PackageJson, Pkg, Provider};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
};
use anyhow::Result;

pub struct YarnProvider {}

impl Provider for YarnProvider {
    fn name(&self) -> &str {
        "yarn"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json") && app.includes_file("yarn.lock"))
    }

    fn pkgs(&self, _app: &App, _env: &Environment) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.yarn")]
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("yarn".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("build").is_some() {
                return Ok(Some("yarn build".to_string()));
            }
        }

        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("start").is_some() {
                return Ok(Some("yarn start".to_string()));
            }
        }

        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                return Ok(Some(format!("node {}", main)));
            }
        }
        if app.includes_file("index.js") {
            return Ok(Some(String::from("node index.js")));
        }

        Ok(None)
    }

    fn get_environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        let mut variables = EnvironmentVariables::default();
        variables.insert("NODE_ENV".to_string(), "production".to_string());

        Ok(variables)
    }
}
