use std::fmt::format;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

const MIX_CACHE_DIR: &'static &str = &"/root/mix";

pub struct ElixirProvider;

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("elixir")])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("mix local.hex --force".to_string());
        install_phase.add_cmd("mix local.rebar --force".to_string());
        install_phase.add_cmd("mix deps.get".to_string());
        Ok(Some(install_phase))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let build_phase = BuildPhase::new("mix compile".to_string());
        Ok(Some(build_phase))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let start_phase = StartPhase::new("mix run --no-halt".to_string());
        Ok(Some(start_phase))
    }
}
