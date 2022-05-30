use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
enum Framework {
    Rails,
    /// No framework could be found
    Vanilla,
}

pub struct RubyProvider {}

impl Provider for RubyProvider {
    fn name(&self) -> &str {
        "ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile") || app.has_match("*.rb"))
    }

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let framework = self.detect_framework(app);
        match framework {
            Framework::Rails => Ok(Some(SetupPhase::new(vec![
                Pkg::new("ruby"),
                Pkg::new("postgresql"),
                Pkg::new("nodejs"),
                Pkg::new("java"),
            ]))),
            Framework::Vanilla => Ok(Some(SetupPhase::new(vec![Pkg::new("ruby")]))),
        }
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        if app.includes_file("Gemfile") {
            Ok(Some(InstallPhase::new(String::from(
                "bundle install --frozen",
            ))))
        } else {
            Ok(None)
        }
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(None)
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if app.includes_file("main.rb") {
            return Ok(Some(StartPhase::new(String::from(
                "bundle exec ruby main.rb",
            ))));
        }
        Ok(None)
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(None)
    }
}

impl RubyProvider {
    fn detect_framework(&self, app: &App) -> Framework {
        if app.includes_file("Rakefile") {
            Framework::Rails
        } else {
            Framework::Vanilla
        }
    }
}
