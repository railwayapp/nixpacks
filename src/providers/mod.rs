use crate::nixpacks::{app::App, environment::Environment, plan::BuildPlan};
use anyhow::Result;

pub mod clojure;
pub mod cobol;
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
pub mod procfile;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod staticfile;
pub mod swift;
pub mod zig;

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, _app: &App, _env: &Environment) -> Result<bool> {
        Ok(false)
    }
    fn get_build_plan(&self, _app: &App, _environment: &Environment) -> Result<Option<BuildPlan>>;
    fn metadata(&self, _app: &App, _env: &Environment) -> Result<ProviderMetadata> {
        Ok(ProviderMetadata::default())
    }
}

#[derive(Default)]
pub struct ProviderMetadata {
    pub values: Option<Vec<String>>,
}

impl ProviderMetadata {
    pub fn from(value_pairs: Vec<(bool, &str)>) -> ProviderMetadata {
        let values = value_pairs
            .into_iter()
            .filter(|(include, _)| *include)
            .map(|(_, value)| (*value).to_owned())
            .collect();

        ProviderMetadata {
            values: Some(values),
        }
    }

    pub fn join_as_comma_separated(&self, provider_name: String) -> String {
        let mut arr = vec![provider_name];
        let mut labels_arr = match &self.values {
            Some(v) => v.clone(),
            _ => Vec::new(),
        };

        arr.append(labels_arr.as_mut());
        arr.join(",")
    }
}

#[test]
fn test_join_as_comma_separated() {
    let metadata = ProviderMetadata::from(vec![
        (true, "test_tag"),
        (false, "test_other_tag"),
        (true, "test_tag_3"),
    ]);

    let tags_str = &metadata.join_as_comma_separated("my_provider".to_string());
    assert_eq!(tags_str, "my_provider,test_tag,test_tag_3");
}
