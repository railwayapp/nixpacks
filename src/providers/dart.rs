use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use serde::Deserialize;

pub const DEFAULT_DART_PKG_NAME: &'static &str = &"dart";

#[derive(Deserialize, Debug)]
pub struct DartPubspec {
    pub name: String,
    pub version: String,
}

pub struct DartProvider {}

impl Provider for DartProvider {
    fn name(&self) -> &str {
        "dart"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("pubspec.yaml"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new(DEFAULT_DART_PKG_NAME)])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_cmd = InstallPhase::new("dart pub get".to_string());
        install_cmd.add_file_dependency("pubspec.yaml".to_string());

        Ok(Some(install_cmd))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let pubspec = DartProvider::get_pubspec(app)?;
        let command = format!("dart compile exe bin/{}.dart", pubspec.name);

        Ok(Some(BuildPhase::new(command)))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let pubspec = DartProvider::get_pubspec(app)?;
        let command = format!("./bin/{}.exe", pubspec.name);

        Ok(Some(StartPhase::new(command)))
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
