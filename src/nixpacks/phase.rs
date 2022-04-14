use serde::{Deserialize, Serialize};

use super::nix::NixConfig;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SetupPhase {
    pub nix_config: NixConfig,
    pub file_dependencies: Option<Vec<String>>,
}

impl SetupPhase {
    pub fn new(nix_config: NixConfig) -> Self {
        Self {
            nix_config,
            file_dependencies: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.file_dependencies.clone() {
            files.push(file);
            self.file_dependencies = Some(files);
        } else {
            self.file_dependencies = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InstallPhase {
    pub cmd: Option<String>,
    pub file_dependencies: Option<Vec<String>>,
}

impl InstallPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            file_dependencies: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.file_dependencies.clone() {
            files.push(file);
            self.file_dependencies = Some(files);
        } else {
            self.file_dependencies = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BuildPhase {
    pub cmd: Option<String>,
    pub file_dependencies: Option<Vec<String>>,
}

impl BuildPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            file_dependencies: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.file_dependencies.clone() {
            files.push(file);
            self.file_dependencies = Some(files);
        } else {
            self.file_dependencies = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StartPhase {
    pub cmd: Option<String>,
}

impl StartPhase {
    pub fn new(cmd: String) -> Self {
        Self { cmd: Some(cmd) }
    }
}
