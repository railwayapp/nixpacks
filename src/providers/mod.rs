use crate::nixpacks::{app::App, environment::Environment, plan::BuildPlan};
use anyhow::Result;

pub mod clojure;
pub mod crystal;
pub mod csharp;
pub mod dart;
pub mod deno;
pub mod elixir;
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
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;
    fn get_build_plan(&self, _app: &App, _environment: &Environment) -> Result<Option<BuildPlan>>;
    fn get_metadata(&self, _app: &App, _env: &Environment) -> Result<ProviderMetadata> {
        Ok(ProviderMetadata::default())
    }
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
