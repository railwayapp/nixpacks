use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    plan::{
        legacy_phase::{LegacyBuildPhase, LegacyInstallPhase, LegacySetupPhase, LegacyStartPhase},
        BuildPlan,
    },
};
use anyhow::Result;

pub mod clojure;
pub mod crystal;
pub mod csharp;
pub mod dart;
pub mod deno;
pub mod fsharp;
pub mod go;
pub mod haskell;
pub mod java;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod staticfile;
pub mod swift;
pub mod zig;

pub trait Provider {
    fn name(&self) -> &str;

    fn detect(&self, app: &App, _env: &Environment) -> Result<DetectResult>;

    fn get_build_plan(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<BuildPlan>> {
        Ok(None)
    }

    fn setup(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<LegacySetupPhase>> {
        Ok(None)
    }
    fn install(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<LegacyInstallPhase>> {
        Ok(None)
    }
    fn build(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<LegacyBuildPhase>> {
        Ok(None)
    }
    fn start(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<LegacyStartPhase>> {
        Ok(None)
    }
    fn static_assets(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<StaticAssets>> {
        Ok(None)
    }
    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
        _metadata: &ProviderMetadata,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(None)
    }
}

pub struct DetectResult {
    pub detected: bool,
    pub metadata: Option<ProviderMetadata>,
}

#[derive(Default)]
pub struct ProviderMetadata {
    pub labels: Option<Vec<String>>,
}

impl ProviderMetadata {
    pub fn from(values: Vec<(bool, &str)>) -> ProviderMetadata {
        let labels = values
            .into_iter()
            .filter(|(include, _)| *include)
            .map(|(_, value)| (*value).to_owned())
            .collect();

        ProviderMetadata {
            labels: Some(labels),
        }
    }

    pub fn join_as_comma_separated(&self, provider_name: String) -> String {
        let mut arr = vec![provider_name];
        let mut labels_arr = match &self.labels {
            Some(v) => v.clone(),
            _ => vec![],
        };

        arr.append(labels_arr.as_mut());
        arr.join(",")
    }

    pub fn has_label(&self, name: &str) -> bool {
        match &self.labels {
            None => false,
            Some(value) => value.contains(&name.to_string()),
        }
    }
}
