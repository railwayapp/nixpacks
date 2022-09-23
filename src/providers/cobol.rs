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

pub struct CobolProvider {}

impl Provider for CobolProvider {
    fn name(&self) -> &str {
        "cobol"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        println!("cobol detect {:?}", app.has_match("*.cbl"));
        Ok(app.has_match("*.cbl"))
    }

    fn get_build_plan(
        &self,
        _app: &App,
        _environment: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::plan::BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("gnu-cobol"), Pkg::new("gcc")]));

        let mut build = Phase::build(Some(format!("cobc -free -x -o index index.cbl")));
        build.depends_on_phase("setup");

        let start = StartPhase::new("./index");

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}
