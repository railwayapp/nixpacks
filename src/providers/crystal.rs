use std::collections::HashMap;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use serde::Deserialize;

// https://github.com/crystal-lang/shards/blob/master/docs/shard.yml.adoc
#[derive(Deserialize, Debug)]
pub struct ShardYaml {
    pub name: String,
    pub targets: HashMap<String, HashMap<String, String>>,
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

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let config = CrystalProvider::get_config(app)?;
        let target_names = config.targets.keys().cloned().collect::<Vec<_>>();
        let start_phase = StartPhase::new(format!(
            "./bin/{}",
            target_names
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Unable to get executable name"))?
        ));

        Ok(Some(start_phase))
    }
}

impl CrystalProvider {
    fn get_config(app: &App) -> Result<ShardYaml> {
        app.read_yaml::<ShardYaml>("shard.yml")
            .context("Reading shard.yml")
    }
}
