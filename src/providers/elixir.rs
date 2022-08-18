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
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DenoTasks {
    pub start: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DenoJson {
    pub tasks: Option<DenoTasks>,
}

pub struct ElixirProvider {}

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup_phase = Phase::setup(Some(vec![Pkg::new("elixir")]));
        plan.add_phase(setup_phase);

        // Install Phase
        let mut install_phase = Phase::install(Some("mix local.hex --force".to_string()));
        install_phase.add_cmd("mix local.rebar --force".to_string());
        install_phase.add_cmd("mix deps.get".to_string());
        plan.add_phase(install_phase);

        // Build Phase
        let build_phase = Phase::build(Some("mix compile".to_string()));
        plan.add_phase(build_phase);

        // Start Phase
        let start_phase = StartPhase::new("mix run --no-halt".to_string());
        plan.set_start_phase(start_phase);

        Ok(Some(plan))
    }
}
