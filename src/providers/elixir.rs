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

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup_phase = Phase::setup(Some(vec![Pkg::new("elixir")]));
        plan.add_phase(setup_phase);

        // Install Phase
        let mut install_phase =
            Phase::install(Some("MIX_ENV=prod mix local.hex --force".to_string()));
        install_phase.add_cmd("MIX_ENV=prod mix local.rebar --force");
        install_phase.add_cmd("MIX_ENV=prod mix deps.get --only prod");
        plan.add_phase(install_phase);

        // Build Phase
        let mut build_phase = Phase::build(Some("MIX_ENV=prod mix compile".to_string()));
        let mix_exs_content = app.read_file("mix.exs")?;

        if mix_exs_content.contains("assets.deploy") {
            build_phase.add_cmd("MIX_ENV=prod mix assets.deploy".to_string());
        }

        if mix_exs_content.contains("postgrex") && mix_exs_content.contains("ecto") {
            build_phase.add_cmd("MIX_ENV=prod mix ecto.migrate");
            build_phase.add_cmd("MIX_ENV=prod mix run priv/repo/seeds.exs");
        }
        plan.add_phase(build_phase);

        // Start Phase
        let start_phase = StartPhase::new("mix phx.server".to_string());
        plan.set_start_phase(start_phase);

        Ok(Some(plan))
    }
}
