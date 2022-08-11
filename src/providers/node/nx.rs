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
pub struct ProjectJson {
    pub targets: Targets,
}
#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Targets {
    pub build: Target,
    pub start: Option<Target>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Target {
    pub executor: String,
    pub options: Options,
    pub configuration: Option<Configuration>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Options {
    #[serde(alias = "outputPath")]
    pub output_path: Option<Value>,
    pub main: Option<Value>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Configuration {
    pub production: Option<Value>,
}
