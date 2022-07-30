use std::path::PathBuf;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DenoTasks {
    pub start: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DenoJson {
    pub tasks: Option<DenoTasks>,
}

pub struct DenoProvider {}

impl Provider for DenoProvider {
    fn name(&self) -> &str {
        "deno"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        let re = Regex::new(
            r##"import .+ from (?:"|'|`)https://deno.land/[^"`']+\.(?:ts|js|tsx|jsx)(?:"|'|`);?"##,
        )
        .unwrap();
        Ok(app.includes_file("deno.json")
            || app.includes_file("deno.jsonc")
            || app.find_match(&re, "**/*.{tsx,ts,js,jsx}")?)
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("deno")])))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        match DenoProvider::get_start_file(app)? {
            Some(start_file) => Ok(Some(BuildPhase::new(format!(
                "deno cache {}",
                start_file
                    .to_str()
                    .context("Failed to convert start_file to string")?
            )))),
            None => Ok(None),
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        // First check for a deno.json and see if we can rip the start command from there
        if app.includes_file("deno.json") {
            let deno_json: DenoJson = app.read_json("deno.json")?;

            if let Some(tasks) = deno_json.tasks {
                if let Some(start) = tasks.start {
                    return Ok(Some(StartPhase::new(start)));
                }
            }
        }

        // Barring that, just try and start the index with sane defaults
        match DenoProvider::get_start_file(app)? {
            Some(start_file) => Ok(Some(StartPhase::new(format!(
                "deno run --allow-all {}",
                start_file
                    .to_str()
                    .context("Failed to convert start_file to string")?
            )))),
            None => Ok(None),
        }
    }
}

impl DenoProvider {
    // Find the first index.ts or index.js file to run
    fn get_start_file(app: &App) -> Result<Option<PathBuf>> {
        // Find the first index.ts or index.js file to run
        let matches = app.find_files("**/index.[tj]s")?;
        let path_to_index = match matches.first() {
            Some(m) => m,
            None => return Ok(None),
        };

        let relative_path_to_index = app.strip_source_path(path_to_index)?;
        Ok(Some(relative_path_to_index))
    }
}
