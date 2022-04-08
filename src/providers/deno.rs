use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::{NixConfig, Pkg},
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

    fn pkgs(&self, _app: &App, _env: &Environment) -> Result<NixConfig> {
        Ok(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.deno"),
        ]))
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        // Find the first index.ts or index.js file to run
        let matches = app.find_files("**/index.[tj]s")?;
        let path_to_index = match matches.first() {
            Some(m) => m.to_string(),
            None => return Ok(None),
        };

        let relative_path_to_index = app.strip_source_path(&path_to_index)?;
        return Ok(Some(format!(
            "deno run --allow-all {}",
            relative_path_to_index
        )));
    }
}
