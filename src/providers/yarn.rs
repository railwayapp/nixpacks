use super::{
    npm::{NpmProvider, PackageJson, DEFAULT_NODE_PKG_NAME},
    Provider,
};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::Pkg,
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

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = NpmProvider::get_nix_node_pkg(&package_json)?;
        let mut yarn_pkg = Pkg::new("yarn");

        // Only override the node package if not the default one
        if node_pkg.name != *DEFAULT_NODE_PKG_NAME {
            yarn_pkg = yarn_pkg.set_override("nodejs", node_pkg.name.as_str());
        }

        Ok(Some(SetupPhase::new(vec![node_pkg, yarn_pkg])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let install_phase =
            InstallPhase::new("yarn install --production=false --frozen-lockfile".to_string());
        Ok(Some(install_phase))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        if NpmProvider::has_script(app, "build")? {
            Ok(Some(BuildPhase::new("yarn build".to_string())))
        } else {
            Ok(None)
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = NpmProvider::get_start_cmd(app)? {
            Ok(Some(StartPhase::new(start_cmd.replace("npm run", "yarn"))))
        } else {
            Ok(None)
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(NpmProvider::get_node_environment_variables()))
    }
}
