// Code relating to NX Monorepos

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, PartialEq, Deserialize)]
struct NxJson {
    #[serde(default)]
    #[serde(alias = "defaultProject")]
    default_project: Option<Value>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Options {
    #[serde(alias = "outputPath")]
    output_path: Option<Value>,
    #[serde(default)]
    main: Option<Value>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Build {
    executor: String,
    options: Options,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct Targets {
    build: Build,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct ProjectJson {
    targets: Targets,
}
