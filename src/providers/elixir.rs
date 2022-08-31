use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
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

        plan.add_variables(EnvironmentVariables::from([(
            "MIX_ENV".to_string(),
            "prod".to_string(),
        )]));

        // Install Phase
        let mut install_phase = Phase::install(Some("mix local.hex --force".to_string()));
        install_phase.add_cmd("mix local.rebar --force");
        install_phase.add_cmd("mix deps.get --only prod");
        plan.add_phase(install_phase);

        // Build Phase
        let mut build_phase = Phase::build(Some("mix compile".to_string()));
        let mix_exs_content = app.read_file("mix.exs")?;

        if mix_exs_content.contains("assets.deploy") {
            build_phase.add_cmd("mix assets.deploy".to_string());
        }

        if mix_exs_content.contains("postgrex") && mix_exs_content.contains("ecto") {
            build_phase.add_cmd("mix ecto.migrate");
            build_phase.add_cmd("mix run priv/repo/seeds.exs");
        }
        plan.add_phase(build_phase);

        // Start Phase
        let start_phase = StartPhase::new("mix phx.server".to_string());
        plan.set_start_phase(start_phase);

        Ok(Some(plan))
    }
}
