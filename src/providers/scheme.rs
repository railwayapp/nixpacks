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

pub struct HauntProvider {}

impl Provider for HauntProvider {
    fn name(&self) -> &str {
        "scheme"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("haunt.scm"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("haunt")]));
        let build = Phase::build(Some("haunt build".to_string()));
        let start = StartPhase::new("haunt serve".to_string());

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        
        Ok(Some(plan))
    }
}
