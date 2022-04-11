use serde::{Deserialize, Serialize};

use super::nix::NixConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupPhase {
    pub file_dependencies: Vec<String>,
    pub nix_config: NixConfig,
}

impl SetupPhase {
    pub fn new(nix_config: NixConfig) -> SetupPhase {
        SetupPhase {
            file_dependencies: Vec::new(),
            nix_config,
        }
    }
}
