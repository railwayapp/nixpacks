use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
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

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("deno")])))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        match DenoProvider::get_start_file(app)? {
            Some(start_file) => Ok(Some(BuildPhase::new(format!("deno cache {}", start_file)))),
            None => Ok(None),
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        match DenoProvider::get_start_file(app)? {
            Some(start_file) => Ok(Some(StartPhase::new(format!(
                "deno run --allow-all {}",
                start_file
            )))),
            None => Ok(None),
        }
    }
}

impl DenoProvider {
    // Find the first index.ts or index.js file to run
    fn get_start_file(app: &App) -> Result<Option<String>> {
        // Find the first index.ts or index.js file to run
        let matches = app.find_files("**/index.[tj]s")?;
        let path_to_index = match matches.first() {
            Some(m) => m.to_string(),
            None => return Ok(None),
        };

        let relative_path_to_index = app.strip_source_path(&path_to_index)?;
        Ok(Some(relative_path_to_index))
    }
}
