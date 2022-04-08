use serde::{Deserialize, Serialize};

use super::{
    environment::EnvironmentVariables,
    nix::{NixConfig},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: String,
    pub nix_config: NixConfig,
    pub install_cmd: Option<String>,
    pub build_cmd: Option<String>,
    pub start_cmd: Option<String>,
    pub variables: EnvironmentVariables,
}
