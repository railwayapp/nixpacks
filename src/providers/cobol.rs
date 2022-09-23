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
use anyhow::{bail, Result};

const COBOL_COMPILE_ARGS: &str = "COBOL_COMPILE_ARGS";
const COBOL_APP_NAME: &str = "COBOL_APP_NAME";

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
            .get_variable(COBOL_COMPILE_ARGS)
            .unwrap_or("-x -o");

        let app_path = CobolProvider::get_app_path(&self, app, environment).unwrap();
        let file_name = app_path.file_stem().unwrap().to_str().unwrap();

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
    fn get_app_path(&self, app: &App, environment: &Environment) -> anyhow::Result<PathBuf> {
        if let Some(app_name) = environment.get_config_variable(COBOL_APP_NAME) {
            if let Ok(matches) = app.find_files(&format!("*{}.cbl", &app_name)) {
                if let Some(path) = matches.first() {
                    return app.strip_source_path(path);
                };
            }
        }

        if app.includes_file("index.cbl") {
            return Ok(PathBuf::from("index.cbl"));
        } else if app.includes_file("./src/index.cbl") {
            return Ok(PathBuf::from("./src/index.cbl"));
        }

        if let Ok(matches) = app.find_files("*.cbl") {
            if let Some(first) = matches.first() {
                return Ok(first.clone());
            }
        }

        bail!(format!("Could not work out COBOL to compile and run please provide the NIXPACKS_{} environment variable", COBOL_APP_NAME));
    }
}
