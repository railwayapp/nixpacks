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
use std::ffi::OsStr;

pub struct ZigProvider;

impl Provider for ZigProvider {
    fn name(&self) -> &'static str {
        "zig"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.has_match("*.zig") || app.has_match("**/*.zig"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("zig")]));

        let mut install = Phase::install(None);
        if app.includes_file(".gitmodules") {
            install.add_cmd("git submodule update --init".to_string());
        }

        let build = Phase::build(Some("zig build -Doptimize=ReleaseSafe".to_string()));

        let start = StartPhase::new(format!(
            "./zig-out/bin/{}",
            app.source
                .file_name()
                .map(OsStr::to_str)
                .map_or("*", Option::unwrap)
        ));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        Ok(Some(plan))
    }
}
