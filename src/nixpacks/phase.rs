use serde::{Deserialize, Serialize};

use super::nix::NixConfig;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SetupPhase {
    pub nix: NixConfig,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,
}

impl SetupPhase {
    pub fn new(nix: NixConfig) -> Self {
        Self {
            nix,
            only_include_files: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.only_include_files.clone() {
            files.push(file);
            self.only_include_files = Some(files);
        } else {
            self.only_include_files = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct InstallPhase {
    pub cmd: Option<String>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,
}

impl InstallPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            only_include_files: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.only_include_files.clone() {
            files.push(file);
            self.only_include_files = Some(files);
        } else {
            self.only_include_files = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct BuildPhase {
    pub cmd: Option<String>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,
}

impl BuildPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            only_include_files: None,
        }
    }

    pub fn add_file_dependency(&mut self, file: String) {
        if let Some(mut files) = self.only_include_files.clone() {
            files.push(file);
            self.only_include_files = Some(files);
        } else {
            self.only_include_files = Some(vec![file]);
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct StartPhase {
    pub cmd: Option<String>,
}

impl StartPhase {
    pub fn new(cmd: String) -> Self {
        Self { cmd: Some(cmd) }
    }
}
