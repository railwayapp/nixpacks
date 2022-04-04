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
        if !app.includes_file("src/index.ts") {
            false;
        }

        let re = Regex::new(r##"(?m)^import .+ from "https://deno.land/[^"]+\.ts";?$"##).unwrap();
        Ok(app.find_match(&re, "**/*.ts"))
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

    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("deno run src/index.ts".to_string()))
    }
}
