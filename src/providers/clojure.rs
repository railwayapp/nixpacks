use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
pub struct ClojureProvider {}

impl Provider for ClojureProvider {
    fn name(&self) -> &str {
        "clojure"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("project.clj"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![
            Pkg::new("leiningen"),
            Pkg::new("jdk8"),
        ])))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new(
            "lein uberjar".to_string(),
        )))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(
            "java $JAVA_OPTS -jar target/uberjar/*standalone.jar".to_string(),
        )))
    }
}
