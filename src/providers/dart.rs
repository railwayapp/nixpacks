use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Result};
use serde::Deserialize;

pub const DEFAULT_DART_PKG_NAME: &str = "dart";

#[derive(Deserialize, Debug)]
pub struct DartPubspec {
    pub name: String,
    pub version: String,
}

pub struct DartProvider {}

impl Provider for DartProvider {
    fn name(&self) -> &'static str {
        "dart"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("pubspec.yaml"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new(DEFAULT_DART_PKG_NAME)]));

        let mut install = Phase::install(Some("dart pub get".to_string()));
        install.add_file_dependency("pubspec.yaml".to_string());

        let pubspec = DartProvider::get_pubspec(app)?;
        let build = Phase::build(Some(format!("dart compile exe bin/{}.dart", pubspec.name)));

        let pubspec = DartProvider::get_pubspec(app)?;
        let start = StartPhase::new(format!("./bin/{}.exe", pubspec.name));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        Ok(Some(plan))
    }
}

impl DartProvider {
    fn get_pubspec(app: &App) -> Result<DartPubspec> {
        app.read_yaml::<DartPubspec>("pubspec.yaml")
            .context("Reading pubspec.yaml")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pubspec() -> Result<()> {
        let pubspec = DartProvider::get_pubspec(&App::new("./examples/dart")?)?;
        assert_eq!(pubspec.name, "console_simple");
        assert_eq!(pubspec.version, "1.0.0");

        Ok(())
    }
}
