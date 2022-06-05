use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;

enum Mode {
    Compiled,
    Script,
}

enum Framework {
    Nerves,
    Phoenix,
    Vanilla,
}

pub struct ElixirProvider {}

const AVAILABLE_ELIXIR_VERSIONS: &[(&str, &str)] = &[
    ("1.9", "elixir_1_9"),
    ("1.10", "elixir_1_10"),
    ("1.11", "elxir_1_11"),
    ("1.12", "elixir_1_12"),
    ("1.13", "elixir"),
];
pub const DEFAULT_ELIXIR_PKG_NAME: &'static &str = &"elixir";

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs") || app.has_match("*.ex") || app.has_match("*.exs"))
    }
}

impl ElixirProvider {
    fn detect_framework(&self, app: &App) -> Framework {
        let phoenix_re = Regex::new(r"\{\:phoenix\,.+\}").unwrap();
        let has_phoenix_dep = app.find_match(&phoenix_re, "mix.exs").unwrap_or(false);

        if has_phoenix_dep && app.has_match("**/*_web/**") {
            return Framework::Phoenix;
        }

        let nerves_re = Regex::new(r"\{\:nerves\,.+\}").unwrap();
        let has_nerves_dep = app.find_match(&nerves_re, "mix.exs").unwrap_or(false);

        if has_nerves_dep {
            return Framework::Nerves;
        }

        Framework::Vanilla
    }

    fn detect_mode(&self, app: &App) -> Mode {
        if app.includes_file("mix.exs") {
            return Mode::Compiled;
        }

        Mode::Script
    }

    fn is_umbrella(&self, app: &App) -> bool {
        app.includes_directory("apps")
    }

    fn is_escript(&self, app: &App) -> bool {
        let escript_re = Regex::new(r"escript\:").unwrap();
        let main_module_re = Regex::new(r"main_module\:").unwrap();
        let has_escript_key = app.find_match(&escript_re, "mix.exs").unwrap_or(false);
        let has_main_module = app.find_match(&main_module_re, "mix.exs").unwrap_or(false);

        has_escript_key && has_main_module
    }

    fn read_mix_exs_if_exists(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("mix.exs") {
            return Ok(Some(app.read_file("mix.exs")?));
        }

        Ok(None)
    }

    fn get_nix_elixir_pkg(mix_exs_contents: Option<String>) -> Result<String> {
        let elixir_nix_pkg = mix_exs_contents
            .as_ref()
            .unwrap()
            .lines()
            .find(|line| line.trim().starts_with("elixir: ~> "))
            .and_then(|version_line| version_line.trim().split_whitespace().nth(1))
            .and_then(|version| version_number_to_pkg(version).ok()?);

        if let Some(nix_pkg) = elixir_nix_pkg {
            return Ok(nix_pkg);
        }

        Ok(DEFAULT_ELIXIR_PKG_NAME.to_string())
    }
}

fn version_number_to_pkg(version: &str) -> Result<Option<String>> {
    let matched_version = AVAILABLE_ELIXIR_VERSIONS
        .iter()
        .find(|(v, _)| v == &version);

    match matched_version {
        Some((_, pkg)) => Ok(Some(pkg.to_string())),
        None => Ok(None),
    }
}
