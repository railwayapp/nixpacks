use serde::{Deserialize, Serialize};

use crate::providers::Pkg;

use super::environment::EnvironmentVariables;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: String,
    pub nixpkgs_archive: Option<String>,
    pub pkgs: Vec<Pkg>,
    pub install_cmd: Option<String>,
    pub build_cmd: Option<String>,
    pub start_cmd: Option<String>,
    pub variables: EnvironmentVariables,
}
