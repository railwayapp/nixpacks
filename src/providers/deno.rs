use super::Provider;
use crate::{nixpacks::app::App, providers::Pkg};
use anyhow::Result;
use regex::Regex;

pub struct DenoProvider {}

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        "deno"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        let re = Regex::new(r##"(?m)^import .+ from "https://deno.land/[^"]+\.ts";?$"##).unwrap();
        app.find_match(&re, "**/*.ts")
    }

    fn pkgs(&self, _app: &App) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.deno")]
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, app: &App) -> Result<Option<String>> {
        // Find the first index.ts or index.js file to run
        let matches = app.find_files("**/index.[tj]s")?;
        let path_to_index = match matches.first() {
            Some(m) => m.to_string(),
            None => return Ok(None),
        };

        return Ok(Some(format!("deno run --allow-all {}", path_to_index)));
    }
}
