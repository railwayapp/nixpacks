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
    pub tags: Option<Vec<String>>,
}

impl ProviderMetadata {
    pub fn from(values: Vec<(bool, &str)>) -> ProviderMetadata {
        let labels = values
            .into_iter()
            .filter(|(include, _)| *include)
            .map(|(_, value)| (*value).to_owned())
            .collect();

        ProviderMetadata { tags: Some(labels) }
    }

    pub fn join_as_comma_separated(&self, provider_name: String) -> String {
        let mut arr = vec![provider_name];
        let mut labels_arr = match &self.tags {
            Some(v) => v.clone(),
            _ => Vec::new(),
        };

        arr.append(labels_arr.as_mut());
        arr.join(",")
    }

    pub fn has_label(&self, name: &str) -> bool {
        match &self.tags {
            None => false,
            Some(value) => value.contains(&name.to_string()),
        }
    }
}

#[test]
fn test_provider_metadata_from() {
    let metadata = ProviderMetadata::from(vec![
        (true, "test_tag"),
        (false, "test_other_tag"),
        (true, "test_tag_3"),
    ]);

    let tags = &metadata.tags.as_ref();
    assert!(tags.is_some());
    assert_eq!(tags.unwrap().len(), 2);
    assert!(&metadata.has_label("test_tag"));
    assert!(&metadata.has_label("test_tag_3"));
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
