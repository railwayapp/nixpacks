use std::path::PathBuf;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Result};
use path_slash::PathBufExt;
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
            r#"import .+ from (?:"|'|`)https://deno.land/[^"`']+\.(?:ts|js|tsx|jsx)(?:"|'|`);?"#,
        )
        .unwrap();
        Ok(app.includes_file("deno.json")
            || app.includes_file("deno.jsonc")
            || app.find_match(&re, "**/*.{ts,tsx,js,jsx}")?)
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup = Phase::setup(Some(vec![Pkg::new("deno")]));
        plan.add_phase(setup);

        if let Some(build_cmd) = DenoProvider::get_build_cmd(app)? {
            let mut build = Phase::build(Some(build_cmd));
            build.depends_on_phase("setup");
            plan.add_phase(build);
        };

        if let Some(start_cmd) = DenoProvider::get_start_cmd(app)? {
            let start = StartPhase::new(start_cmd);
            plan.set_start_phase(start);
        }

        Ok(Some(plan))
    }
}

impl DenoProvider {
    fn get_build_cmd(app: &App) -> Result<Option<String>> {
        if let Some(start_file) = DenoProvider::get_start_file(app)? {
            Ok(Some(format!(
                "deno cache {}",
                start_file
                    .to_slash()
                    .context("Failed to convert start_file to slash_path")?
            )))
        } else {
            Ok(None)
        }
    }

    fn get_start_cmd(app: &App) -> Result<Option<String>> {
        // First check for a deno.{json,jsonc} and see if we can rip the start command from there
        if app.includes_file("deno.json") || app.includes_file("deno.jsonc") {
            let deno_json: DenoJson = app
                .read_json("deno.json")
                .or_else(|_| app.read_jsonc("deno.jsonc"))?;

            if let Some(tasks) = deno_json.tasks {
                if let Some(start) = tasks.start {
                    return Ok(Some(start));
                }
            }
        }

        // Barring that, just try and start the index file with sane defaults
        match DenoProvider::get_start_file(app)? {
            Some(start_file) => Ok(Some(format!(
                "deno run --allow-all {}",
                start_file
                    .to_slash()
                    .context("Failed to convert start_file to slash_path")?
            ))),
            None => Ok(None),
        }
    }

    // Find the first index.{ts,tsx,js,jsx} file to run
    fn get_start_file(app: &App) -> Result<Option<PathBuf>> {
        let matches = app.find_files("**/index.{ts,tsx,js,jsx}")?;
        let path_to_index = match matches.first() {
            Some(m) => m,
            None => return Ok(None),
        };

        let relative_path_to_index = app.strip_source_path(path_to_index)?;
        Ok(Some(relative_path_to_index))
    }
}
