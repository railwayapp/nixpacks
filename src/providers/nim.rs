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
use anyhow::{Context, Result};

pub const DEFAULT_NIM_PKG_NAME: &str = "nim";

pub struct NimProvider {}

impl Provider for NimProvider {
    fn name(&self) -> &str {
        "nim"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.find_files("*.nimble")?.len() > 0)
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new(DEFAULT_NIM_PKG_NAME)]));

        let mut install = Phase::install(Some("nimble install -dy".to_string()));
        let nimble = NimProvider::get_nimble(app)?;
        install.add_file_dependency(&nimble);

        let build = Phase::build(Some("nimble build -d:release".to_string()));

        let start = StartPhase::new(format!("./{}.exe", nimble.replace(".nimble", "")));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        Ok(Some(plan))
    }
}

impl NimProvider {
    fn get_nimble(app: &App) -> Result<String> {
        app.find_files("*.nimble")?
            .first()
            .cloned()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .context("No nimble file found")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_nimble() -> Result<()> {
        let nimble = NimProvider::get_nimble(&App::new("./examples/nim")?)?;
        assert_eq!(nimble, "nim.nimble");

        Ok(())
    }
}
