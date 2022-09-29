use std::{path::PathBuf, str::FromStr};

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
use anyhow::Result;
use path_slash::PathBufExt;

const COBOL_COMPILE_ARGS: &str = "COBOL_COMPILE_ARGS";
const COBOL_APP_NAME: &str = "COBOL_APP_NAME";
const DEFAULT_COBOL_COMPILE_ARGS: &str = "-x -o";

pub struct CobolProvider {}

impl Provider for CobolProvider {
    fn name(&self) -> &str {
        "cobol"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.has_match("*.cbl"))
    }

    fn get_build_plan(
        &self,
        app: &App,
        environment: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::plan::BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("gnu-cobol"), Pkg::new("gcc")]));

        let compile_args = environment
            .get_config_variable(COBOL_COMPILE_ARGS)
            .unwrap_or_else(|| DEFAULT_COBOL_COMPILE_ARGS.to_string());

        let app_path = CobolProvider::get_app_path(self, app, environment);

        let file_name = app_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();

        let mut build = Phase::build(Some(format!(
            "cobc {} {} {}",
            compile_args,
            file_name,
            app_path.as_os_str().to_str().unwrap()
        )));
        build.depends_on_phase("setup");

        let start = StartPhase::new(format!("./{}", file_name));

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}

impl CobolProvider {
    fn get_app_path(&self, app: &App, environment: &Environment) -> PathBuf {
        if let Some(app_path) = environment
            .get_config_variable(COBOL_APP_NAME)
            .and_then(|app_name| {
                Some(
                    app.find_files(&format!("*{}.cbl", &app_name))
                        .unwrap_or_default(),
                )
            })
            .and_then(|matches| {
                if let Some(matched) = matches.first() {
                    Some(matched.clone())
                } else {
                    None
                }
            })
            .and_then(|absolute_path| {
                Some(app.strip_source_path(&absolute_path).unwrap_or_default())
            })
            .and_then(|relative_path| CobolProvider::normalized_path(&relative_path))
        {
            return app_path;
        }

        if let Ok(matches) = app.find_files("*index.cbl") {
            if let Some(first) = matches.first() {
                if let Ok(relative_path) = app.strip_source_path(first) {
                    if let Some(normalized_path) = CobolProvider::normalized_path(&relative_path) {
                        return normalized_path;
                    }
                }
            }
        }

        if let Ok(matches) = app.find_files("*.cbl") {
            if let Some(first) = matches.first() {
                if let Ok(relative_path) = app.strip_source_path(first) {
                    if let Some(normalized_path) = CobolProvider::normalized_path(&relative_path) {
                        return normalized_path;
                    }
                }
            }
        }

        PathBuf::from("./")
    }

    fn normalized_path(path: &PathBuf) -> Option<PathBuf> {
        if let Some(normalized_path) = path.to_slash() {
            let path_string = PathBuf::from_str(normalized_path.to_string().as_str());

            if let Ok(path) = path_string {
                return Some(path);
            }
        }
        None
    }
}
