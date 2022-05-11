use std::collections::HashMap;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ShardYaml {
    pub name: String,
    pub targets: HashMap<String, String>,
}

pub struct CrystalProvider {}

impl Provider for CrystalProvider {
    fn name(&self) -> &str {
        "crystal"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("shard.yml"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![
            Pkg::new("crystal"),
            Pkg::new("shards"),
        ])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        Ok(Some(InstallPhase::new("shards install".to_string())))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new("shards build --release".to_string())))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(format!("./bin/crystal"))))
    }
}
