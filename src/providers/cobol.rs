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
    fn name(&self) -> &'static str {
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
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();

        let mut build = Phase::build(Some(format!(
            "cobc {} {} {}",
            compile_args,
            file_name,
            app_path.as_os_str().to_str().unwrap()
        )));
        build.depends_on_phase("setup");

        let start = StartPhase::new(format!("./{file_name}"));

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}

impl CobolProvider {
    fn get_app_path(&self, app: &App, environment: &Environment) -> PathBuf {
        if let Some(app_name) = environment.get_config_variable(COBOL_APP_NAME) {
            if let Some(file_path) =
                CobolProvider::find_first_file(app, &format!("*{}.cbl", &app_name))
            {
                return file_path;
            }
        }

        if let Some(path) = CobolProvider::find_first_file(app, "*index.cbl") {
            return path;
        }
        if let Some(path) = CobolProvider::find_first_file(app, "*.cbl") {
            return path;
        }

        PathBuf::from("./")
    }

    fn find_first_file(app: &App, pattern: &str) -> Option<PathBuf> {
        app.find_files(pattern)
            .unwrap_or_default()
            .first()
            .map(|absolute_path| app.strip_source_path(absolute_path).unwrap_or_default())
            .and_then(|relative_path| CobolProvider::normalized_path(&relative_path))
    }

    fn normalized_path(path: &PathBuf) -> Option<PathBuf> {
        path.to_slash().and_then(|normalized_path| {
            PathBuf::from_str(normalized_path.as_ref())
                .map(Some)
                .unwrap_or(None)
        })
    }
}
