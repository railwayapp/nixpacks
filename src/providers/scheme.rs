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
    fn name(&self) -> &'static str {
        "scheme"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("haunt.scm"))
    }

    fn get_build_plan(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("haunt"), Pkg::new("guile")]));
        let mut build = Phase::build(Some("haunt build".to_string()));
        build.depends_on_phase("setup");
        // In production, init.scm should run "haunt serve"
        // However, "haunt serve" doesn't terminate on its own, which the tests depend on
        // So for the example, init.scm simply logs to the console
        let start = StartPhase::new("guile init.scm --auto-compile".to_string());

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}
