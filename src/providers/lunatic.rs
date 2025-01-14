use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use crate::providers::rust::RustProvider;
use anyhow::Result;
use regex::Regex;

pub struct LunaticProvider {}

impl Provider for LunaticProvider {
    fn name(&self) -> &'static str {
        "lunatic"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        if !app.includes_file("Cargo.toml") {
            return Ok(false);
        }

        let re_runner = Regex::new(r#"runner\s*=\s*"lunatic""#).expect("BUG: Broken regex");
        app.find_match(&re_runner, ".cargo/config.toml")
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = LunaticProvider::get_setup(app, env)?;
        let build = LunaticProvider::get_build(app, env)?;
        let start = LunaticProvider::get_start(app, env)?;

        let plan = BuildPlan::new(&vec![setup, build], start);

        Ok(Some(plan))
    }
}

impl LunaticProvider {
    fn get_setup(app: &App, env: &Environment) -> Result<Phase> {
        let mut setup = RustProvider::get_setup(app, env)?;

        if let Some(pkgs) = &mut setup.nix_pkgs {
            (*pkgs).push("lunatic".into());
        }

        Ok(setup)
    }

    fn get_build(app: &App, env: &Environment) -> Result<Phase> {
        RustProvider::get_build(app, env)
    }

    fn get_start(app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        match RustProvider::get_start(app, env)? {
            Some(start_phase) => match start_phase.cmd {
                Some(bin) => Ok(Some(StartPhase::new(format!("lunatic {bin}")))),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }
}
