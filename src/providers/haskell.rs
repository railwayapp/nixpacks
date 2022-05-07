use std::collections::BTreeMap;

use serde::Deserialize;

use crate::nixpacks::{phase::{SetupPhase, InstallPhase, BuildPhase, StartPhase}, nix::Pkg};

use super::Provider;

pub struct HaskellStackProvider {}

impl Provider for HaskellStackProvider {
    fn name(&self) -> &str {
        "haskell_stack"
    }

    fn detect(&self, app: &crate::nixpacks::app::App, _env: &crate::nixpacks::environment::Environment) -> anyhow::Result<bool> {
        Ok(app.includes_file("stack.yaml") && app.includes_file("package.yaml"))
    }

    fn setup(&self, _app: &crate::nixpacks::app::App, _env: &crate::nixpacks::environment::Environment) -> anyhow::Result<Option<crate::nixpacks::phase::SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("stack")])))
    }

    fn install(&self, _app: &crate::nixpacks::app::App, _env: &crate::nixpacks::environment::Environment) -> anyhow::Result<Option<crate::nixpacks::phase::InstallPhase>> {
        Ok(Some(InstallPhase::new("stack setup".to_string())))
    }

    fn build(&self, _app: &crate::nixpacks::app::App, _env: &crate::nixpacks::environment::Environment) -> anyhow::Result<Option<crate::nixpacks::phase::BuildPhase>> {
        Ok(Some(BuildPhase::new("stack build".to_string())))
    }

    fn start(&self, app: &crate::nixpacks::app::App, _env: &crate::nixpacks::environment::Environment) -> anyhow::Result<Option<crate::nixpacks::phase::StartPhase>> {
        let package: HaskellStackPackageYaml = app.read_yaml("package.yaml")?;
        let exe_names: Vec<String> = package.executables.keys().cloned().collect();
        Ok(Some(StartPhase::new(format!("stack exec {}", exe_names.get(0).ok_or(anyhow::anyhow!("Failed to get executable name"))?).to_string())))
    }
}

#[derive(Deserialize)]
struct HaskellStackPackageYaml {
    pub executables: BTreeMap<String, HaskellStackExecutableDefinition>,
}

#[derive(Deserialize)]
struct HaskellStackExecutableDefinition {
}