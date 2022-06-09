use serde::{Deserialize, Serialize};

use super::{
    images::{DEBIAN_SLIM_IMAGE, DEFAULT_BASE_IMAGE},
    nix::pkg::Pkg,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetupPhase {
    pub pkgs: Vec<Pkg>,
    pub archive: Option<String>,
    pub libraries: Option<Vec<String>>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,

    #[serde(rename = "baseImage")]
    pub base_image: String,
}

impl SetupPhase {
    pub fn new(pkgs: Vec<Pkg>) -> Self {
        Self {
            pkgs,
            libraries: None,
            archive: None,
            only_include_files: None,
            base_image: DEFAULT_BASE_IMAGE.to_string(),
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

    pub fn add_pkgs(&mut self, new_pkgs: &mut Vec<Pkg>) {
        self.pkgs.append(new_pkgs);
    }

    pub fn set_archive(&mut self, archive: String) {
        self.archive = Some(archive);
    }

    pub fn add_library(&mut self, lib: String) {
        if let Some(mut libraries) = self.libraries.clone() {
            libraries.push(lib);
            self.libraries = Some(libraries);
        } else {
            self.libraries = Some(vec![lib]);
        }
    }
}

impl Default for SetupPhase {
    fn default() -> Self {
        Self {
            pkgs: Default::default(),
            libraries: Default::default(),
            archive: Default::default(),
            only_include_files: Default::default(),
            base_image: DEFAULT_BASE_IMAGE.to_string(),
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct InstallPhase {
    pub cmd: Option<String>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,

    pub paths: Option<Vec<String>>,
}

impl InstallPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            only_include_files: None,
            paths: None,
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

    pub fn add_path(&mut self, path: String) {
        if let Some(mut paths) = self.paths.clone() {
            paths.push(path);
            self.paths = Some(paths);
        } else {
            self.paths = Some(vec![path]);
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

    #[serde(rename = "runImage")]
    pub run_image: Option<String>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,
}

impl StartPhase {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd: Some(cmd),
            run_image: None,
            only_include_files: None,
        }
    }

    pub fn run_in_image(&mut self, image_name: String) {
        self.run_image = Some(image_name);
    }

    pub fn run_in_default_image(&mut self) {
        self.run_image = Some(DEFAULT_BASE_IMAGE.to_string());
    }

    pub fn run_in_slim_image(&mut self) {
        self.run_image = Some(DEBIAN_SLIM_IMAGE.to_string());
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
