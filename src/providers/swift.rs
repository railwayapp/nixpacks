use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub struct SwiftProvider {}

impl Provider for SwiftProvider {
    fn name(&self) -> &str {
        "swift"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Package.swift")
            || (app.includes_file("Package.swift") && app.includes_file("Package.resolved")))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let pkg = Pkg::new("swift");

        Ok(Some(SetupPhase::new(vec![pkg])))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new(
            "swift build -c release --static-swift-stdlib".to_string(),
        )))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let raw_paths = app.find_files("Sources/**/main.swift")?;
        let paths = raw_paths
            .iter()
            .filter(|&path| !path.to_string_lossy().contains(".build"))
            .collect::<Vec<_>>();

        let path = match paths.first() {
            Some(path) => path.to_string_lossy().to_string(),
            None => return Ok(None),
        };

        let mut names = path.split('/').collect::<Vec<_>>();

        // Safe to unwrap, because the path was filtered by the glob expression
        let pos = names.iter().position(|&n| n == "Sources").unwrap();

        names.drain(0..pos);

        Ok(Some(StartPhase::new(format!(
            "./.build/release/{}",
            names[1]
        ))))
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        let mut variables = EnvironmentVariables::default();

        // https://stackoverflow.com/a/62271027
        variables.insert("DEBIAN_FRONTEND".to_string(), "noninteractive".to_string());

        Ok(Some(variables))
    }
}
