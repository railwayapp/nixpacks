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
const ELIXIR_NIXPKGS_ARCHIVE: &str = "c5702bd28cbde41a191a9c2a00501f18941efbd0";

pub struct ElixirProvider {}

impl Provider for ElixirProvider {
    fn name(&self) -> &'static str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = self.setup(app, env)?.unwrap_or_default();

        let mut plan = BuildPlan::default();
        plan.add_variables(ElixirProvider::default_elixir_environment_variables());
        plan.add_phase(setup);

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
            build_phase.add_cmd("mix ecto.setup");
        }

        plan.add_phase(build_phase);

        // Start Phase
        let start_phase = StartPhase::new("mix phx.server".to_string());
        plan.set_start_phase(start_phase);

        Ok(Some(plan))
    }
}

impl ElixirProvider {
    fn setup(&self, app: &App, env: &Environment) -> Result<Option<Phase>> {
        let elixir_pkg = ElixirProvider::get_nix_elixir_package(app, env)?;
        // TODO should try to extract and optionally set the OTP version
        let mut setup = Phase::setup(Some(vec![elixir_pkg]));
        setup.set_nix_archive(ELIXIR_NIXPKGS_ARCHIVE.to_string());

        // Many Elixir packages need some C headers to be available
        setup.add_pkgs_libs(vec!["stdenv.cc.cc.lib".to_string()]);
        setup.add_nix_pkgs(&[Pkg::new("gcc")]);

        Ok(Some(setup))
    }

    fn default_elixir_environment_variables() -> EnvironmentVariables {
        let var_map = vec![
            ("MIX_ENV", "prod"),
            // required to avoid the following error:
            // warning: the VM is running with native name encoding of latin1 which may cause Elixir to malfunction as it expects utf8. Please ensure your locale is set to UTF-8 (which can be verified by running "locale" in your shell) or set the ELIXIR_ERL_OPTIONS="+fnu" environment variable
            ("ELIXIR_ERL_OPTIONS", "+fnu"),
        ];

        let mut env_vars = EnvironmentVariables::new();

        for (key, value) in var_map {
            env_vars.insert(key.to_owned(), value.to_owned());
        }

        env_vars
    }

    fn get_nix_elixir_package(app: &App, env: &Environment) -> Result<Pkg> {
        fn as_default(v: Option<Match>) -> &str {
            match v {
                Some(m) => m.as_str(),
                None => "_",
            }
        }

        let mix_exs_content = app.read_file("mix.exs")?;
        let custom_version = env.get_config_variable("ELIXIR_VERSION");

        let mix_elixir_version_regex = Regex::new(r"(elixir:[\s].*[> ])([0-9|\.]*)")?;

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

        // TODO next, let's try .tool-version

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
            ("1", "13") => Ok(Pkg::new("elixir_1_13")),
            ("1", "14") => Ok(Pkg::new("elixir_1_14")),
            ("1", "15") => Ok(Pkg::new("elixir_1_15")),
            ("1", "16") => Ok(Pkg::new("elixir_1_16")),
            ("1", "17") => Ok(Pkg::new("elixir_1_17")),
            _ => Ok(Pkg::new(DEFAULT_ELIXIR_PKG_NAME)),
        }
    }
}
