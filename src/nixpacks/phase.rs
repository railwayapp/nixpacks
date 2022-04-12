use serde::{Deserialize, Serialize};

use super::nix::NixConfig;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SetupPhase {
    pub file_dependencies: Vec<String>,
    pub nix_config: NixConfig,
}

impl SetupPhase {
    pub fn new(nix_config: NixConfig) -> Self {
        Self {
            file_dependencies: Vec::new(),
            nix_config,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InstallPhase {
    pub cmd: Option<String>,
    pub file_dependencies: Vec<String>,
}

impl InstallPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            file_dependencies: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BuildPhase {
    pub cmd: Option<String>,
    pub file_dependencies: Vec<String>,
}

impl BuildPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            file_dependencies: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StartPhase {
    pub cmd: Option<String>,
}

impl StartPhase {
    pub fn new(cmd: String) -> Self {
        Self { cmd: Some(cmd) }
    }
}
