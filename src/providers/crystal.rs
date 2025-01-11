use std::collections::HashMap;

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
use serde::Deserialize;

// https://github.com/crystal-lang/shards/blob/master/docs/shard.yml.adoc
#[derive(Deserialize, Debug)]
pub struct ShardYaml {
    pub name: String,
    pub targets: HashMap<String, HashMap<String, String>>,
}

pub struct CrystalProvider {}

impl Provider for CrystalProvider {
    fn name(&self) -> &'static str {
        "crystal"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("shard.yml"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("crystal"), Pkg::new("shards")]));
        let install = Phase::install(Some("shards install".to_string()));
        let build = Phase::build(Some("shards build".to_string()));

        let config = CrystalProvider::get_config(app)?;
        let target_names = config.targets.keys().cloned().collect::<Vec<_>>();
        let start = StartPhase::new(format!(
            "./bin/{}",
            target_names
                .first()
                .ok_or_else(|| anyhow::anyhow!("Unable to get executable name"))?
        ));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        Ok(Some(plan))
    }
}

impl CrystalProvider {
    fn get_config(app: &App) -> Result<ShardYaml> {
        app.read_yaml::<ShardYaml>("shard.yml")
            .context("Reading shard.yml")
    }
}
