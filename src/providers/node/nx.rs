// Code relating to NX Monorepos

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct NxJson {
    #[serde(default)]
    #[serde(alias = "defaultProject")]
    pub default_project: Option<Value>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Options {
    #[serde(alias = "outputPath")]
    pub output_path: Option<Value>,
    #[serde(default)]
    pub main: Option<Value>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Build {
    pub executor: String,
    pub options: Options,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Targets {
    pub build: Build,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct ProjectJson {
    pub targets: Targets,
}
