use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub struct JavaProvider {}

impl Provider for JavaProvider {
    fn name(&self) -> &str {
        "Java"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("pom.xml")
            || app.includes_directory("pom.atom")
            || app.includes_directory("pom.clj")
            || app.includes_directory("pom.groovy")
            || app.includes_file("pom.rb")
            || app.includes_file("pom.scala")
            || app.includes_file("pom.yaml")
            || app.includes_file("pom.yml"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(None)
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        Ok(None)
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(None)
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(None)
    }
}

impl JavaProvider {}
