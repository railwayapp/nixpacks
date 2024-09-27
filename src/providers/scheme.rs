use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
}

pub struct HauntProvider

impl Provider for HauntProvider {
    fn name(&self) -> &str {
        "scheme"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("haunt.scm"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup = Phase::setup(Some(vec![Pkg::new("haunt")]));
        plan.add_phase(setup);
        let build = self.get_build();
        plan.add_phase(build);
        let start = self.get_start();
        // plan.add_phase(start);
        plan.set_start_phase(start);
        
        Ok(Some(plan))
    }
}

impl HauntProvider {
    fn get_build(&self, _app: &App, _env: &Environment) -> Phase {
        Phase::build(Some("haunt build".into()))
    }

    fn get_start(&self, _app: &App, _env: &Environment) -> StartPhase {
        let mut phase = StartPhase::new("haunt serve");
        phase
    }
}