use super::{
    npm::{NpmProvider, PackageJson},
    Provider,
};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
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

    fn pkgs(&self, app: &App, _env: &Environment) -> Result<NixConfig> {
        let node_pkg = NpmProvider::get_nix_node_pkg(&app.read_json("package.json")?)?;
        Ok(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.yarn").set_override("nodejs", node_pkg.name.as_str()),
        ]))
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("yarn install --frozen-lockfile".to_string()))
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
        Ok(NpmProvider::get_node_environment_variables())
    }
}
