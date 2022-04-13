use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::{NixConfig, Pkg},
    phase::{SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;

pub struct DenoProvider {}

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        "deno"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        let re = Regex::new(r##"(?m)^import .+ from "https://deno.land/[^"]+\.ts";?$"##).unwrap();
        app.find_match(&re, "**/*.ts")
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<SetupPhase> {
        Ok(SetupPhase::new(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.deno"),
        ])))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<StartPhase> {
        // Find the first index.ts or index.js file to run
        let matches = app.find_files("**/index.[tj]s")?;
        let path_to_index = match matches.first() {
            Some(m) => m.to_string(),
            None => return Ok(StartPhase::default()),
        };

        let relative_path_to_index = app.strip_source_path(&path_to_index)?;
        return Ok(StartPhase::new(format!(
            "deno run --allow-all {}",
            relative_path_to_index
        )));
    }
}
