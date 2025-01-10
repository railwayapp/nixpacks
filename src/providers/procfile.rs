use std::collections::HashMap;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Ok, Result};

pub struct ProcfileProvider {}

impl Provider for ProcfileProvider {
    fn name(&self) -> &'static str {
        "procfile"
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        if let Some(release_cmd) = ProcfileProvider::get_release_cmd(app)? {
            let mut release = Phase::new("release");
            release.depends_on = Some(vec![
                "setup".to_owned(),
                "install".to_owned(),
                "build".to_owned(),
            ]);
            release.cmds = Some(vec!["...".to_string(), release_cmd]);
            plan.add_phase(release);
        };

        if let Some(start_cmd) = ProcfileProvider::get_start_cmd(app)? {
            let start_phase = StartPhase::new(start_cmd);
            plan.set_start_phase(start_phase);
        }

        Ok(Some(plan))
    }
}

impl ProcfileProvider {
    fn get_start_cmd(app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let mut procfile: HashMap<String, String> =
                app.read_yaml("Procfile").context("Reading Procfile")?;
            procfile.remove("release");

            if procfile.is_empty() {
                Ok(None)
            } else if let Some(cmd) = procfile.get("web") {
                Ok(Some(cmd.to_string()))
            } else if let Some(cmd) = procfile.get("worker") {
                Ok(Some(cmd.to_string()))
            } else {
                let mut processes: Vec<_> = procfile.iter().collect();
                processes.sort_by_key(|&(key, _)| key);
                let process = processes[0].1.to_string();
                Ok(Some(process))
            }
        } else {
            Ok(None)
        }
    }

    fn get_release_cmd(app: &App) -> Result<Option<String>> {
        if app.includes_file("Procfile") {
            let procfile: HashMap<String, String> =
                app.read_yaml("Procfile").context("Reading Procfile")?;
            if let Some(release) = procfile.get("release") {
                Ok(Some(release.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
