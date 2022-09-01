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
use regex::{Match, Regex};
const DEFAULT_ELIXIR_PKG_NAME: &str = "elixir";

pub struct ElixirProvider {}

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let elixir_pkg = ElixirProvider::get_nix_elixir_package(app, env)?;

        plan.add_variables(EnvironmentVariables::from([(
            "MIX_ENV".to_string(),
            "prod".to_string(),
        )]));

        let setup_phase = Phase::setup(Some(vec![elixir_pkg]));
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

impl ElixirProvider {
    fn get_nix_elixir_package(app: &App, env: &Environment) -> Result<Pkg> {
        fn as_default(v: Option<Match>) -> &str {
            match v {
                Some(m) => m.as_str(),
                None => "_",
            }
        }

        let mix_exs_content = app.read_file("mix.exs")?;
        let custom_version = env.get_config_variable("ELIXIR_VERSION");

        let mix_elixir_version_regex = Regex::new(r#"(elixir:[\s].*[> ])([0-9|\.]*)"#)?;

        // If not from env variable, get it from the .elixir-version file then try to parse from mix.exs
        let custom_version = if custom_version.is_some() {
            custom_version
        } else if custom_version.is_none() && app.includes_file(".elixir-version") {
            Some(app.read_file(".elixir-version")?)
        } else {
            mix_elixir_version_regex
                .captures(&mix_exs_content)
                .map(|c| c.get(2).unwrap().as_str().to_owned())
        };

        // If it's still none, return default
        if custom_version.is_none() {
            return Ok(Pkg::new(DEFAULT_ELIXIR_PKG_NAME));
        }
        let custom_version = custom_version.unwrap();

        // Regex for reading Elixir versions (e.g. 1.8 or 1.12)
        let elixir_version_regex =
            Regex::new(r#"^(?:[\sa-zA-Z-"']*)(\d*)(?:\.*)(\d*)(?:\.*\d*)(?:["']?)$"#)?;

        // Capture matches
        let matches = elixir_version_regex.captures(custom_version.as_str().trim());

        // If no matches, just use default
        if matches.is_none() {
            return Ok(Pkg::new(DEFAULT_ELIXIR_PKG_NAME));
        }
        let matches = matches.unwrap();
        let parsed_version = (as_default(matches.get(1)), as_default(matches.get(2)));

        // Match major and minor versions
        match parsed_version {
            ("1", "9") => Ok(Pkg::new("elixir_1_9")),
            ("1", "10") => Ok(Pkg::new("elixir_1_10")),
            ("1", "11") => Ok(Pkg::new("elixir_1_11")),
            ("1", "12") => Ok(Pkg::new("elixir_1_12")),
            _ => Ok(Pkg::new(DEFAULT_ELIXIR_PKG_NAME)),
        }
    }
}
