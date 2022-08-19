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

pub struct ElixirProvider {}

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs"))
    }

    fn get_build_plan(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup_phase = Phase::setup(Some(vec![Pkg::new("elixir")]));
        plan.add_phase(setup_phase);

        // Install Phase
        let mut install_phase = Phase::install(Some("mix local.hex --force".to_string()));
        install_phase.add_cmd("mix local.rebar --force");
        install_phase.add_cmd("mix deps.get --only prod");
        plan.add_phase(install_phase);

        // Build Phase
        let mut build_phase = Phase::build(Some("mix compile".to_string()));
        build_phase.add_cmd("mix assets.deploy".to_string());
        plan.add_phase(build_phase);

        // TODO: Detect if this needs to be run
        // MIX_ENV=prod mix ecto.migrate

        // Start Phase
        let start_phase = StartPhase::new("mix phx.server".to_string());
        plan.set_start_phase(start_phase);

        Ok(Some(plan))
    }
}
