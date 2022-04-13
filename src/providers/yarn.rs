use super::{
    npm::{NpmProvider, PackageJson},
    Provider,
};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
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

    fn setup(&self, app: &App, _env: &Environment) -> Result<SetupPhase> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = NpmProvider::get_nix_node_pkg(&package_json)?;

        Ok(SetupPhase::new(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.yarn").set_override("nodejs", node_pkg.name.as_str()),
        ])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<InstallPhase> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let mut install_phase = InstallPhase::new("yarn install --frozen-lockfile".to_string());

        // When install deps for a monorepo, we need all workspace package.json files
        if package_json.workspaces.is_none() {
            // Installing node modules only depends on package.json and lock file
            install_phase.add_file_dependency("package.json".to_string());
            install_phase.add_file_dependency("yarn.lock".to_string());
        }

        Ok(install_phase)
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<BuildPhase> {
        if NpmProvider::has_script(app, "build")? {
            Ok(BuildPhase::new("yarn build".to_string()))
        } else {
            Ok(BuildPhase::default())
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<StartPhase> {
        if let Some(start_cmd) = NpmProvider::get_start_cmd(app)? {
            Ok(StartPhase::new(start_cmd.replace("npm run", "yarn")))
        } else {
            Ok(StartPhase::default())
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        Ok(NpmProvider::get_node_environment_variables())
    }
}
