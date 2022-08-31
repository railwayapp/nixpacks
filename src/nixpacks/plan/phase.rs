use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    images::{DEBIAN_SLIM_IMAGE, DEFAULT_BASE_IMAGE},
    nix::pkg::Pkg,
};

use super::topological_sort::TopItem;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Phase {
    pub name: String,

    #[serde(rename = "dependsOn")]
    pub depends_on: Option<Vec<String>>,

    #[serde(rename = "nixPackages")]
    pub nix_pkgs: Option<Vec<Pkg>>,

    #[serde(rename = "nixLibraries")]
    pub nix_libraries: Option<Vec<String>>,

    #[serde(rename = "nixpacksArchive")]
    pub nixpacks_archive: Option<String>,

    #[serde(rename = "aptPackages")]
    pub apt_pkgs: Option<Vec<String>>,

    #[serde(rename = "commands")]
    pub cmds: Option<Vec<String>>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,

    #[serde(rename = "cacheDirectories")]
    pub cache_directories: Option<Vec<String>>,

    pub paths: Option<Vec<String>>,
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

impl TopItem for Phase {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_dependencies(&self) -> &[String] {
        match &self.depends_on {
            Some(depends_on) => depends_on.as_slice(),
            None => &[],
        }
    }
}

impl Phase {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn prefix_name(&mut self, prefix: &str) {
        self.name = format!("{}:{}", prefix, self.name);
    }

    /// Shortcut for creating a setup phase from a list of nix packages.
    pub fn setup(pkgs: Option<Vec<Pkg>>) -> Self {
        Self {
            nix_pkgs: pkgs,
            name: "setup".to_string(),
            ..Default::default()
        }
    }

    /// Shortcut for creating an install phase from a command
    pub fn install(cmd: Option<String>) -> Self {
        Self {
            name: "install".to_string(),
            cmds: cmd.map(|cmd| vec![cmd]),
            depends_on: Some(vec!["setup".to_string()]),
            ..Default::default()
        }
    }

    /// Shortcut for creating a build phase from a command
    pub fn build(cmd: Option<String>) -> Self {
        Self {
            name: "build".to_string(),
            cmds: cmd.map(|cmd| vec![cmd]),
            depends_on: Some(vec!["install".to_string()]),
            ..Default::default()
        }
    }

    pub fn uses_nix(&self) -> bool {
        !self.nix_pkgs.clone().unwrap_or_default().is_empty()
            || !self.nix_libraries.clone().unwrap_or_default().is_empty()
    }

    pub fn depends_on_phase<S: Into<String>>(&mut self, name: S) {
        self.depends_on = Some(add_to_option_vec(self.depends_on.clone(), name.into()));
    }

    pub fn add_nix_pkgs(&mut self, new_pkgs: Vec<Pkg>) {
        self.nix_pkgs = Some(add_multiple_to_option_vec(self.nix_pkgs.clone(), new_pkgs));
    }

    pub fn add_pkgs_libs(&mut self, new_libraries: Vec<String>) {
        self.nix_libraries = Some(add_multiple_to_option_vec(
            self.nix_libraries.clone(),
            new_libraries,
        ));
    }

    pub fn add_apt_pkgs(&mut self, new_pkgs: Vec<String>) {
        self.apt_pkgs = Some(add_multiple_to_option_vec(self.apt_pkgs.clone(), new_pkgs));
    }

    pub fn add_cmd<S: Into<String>>(&mut self, cmd: S) {
        self.cmds = Some(add_to_option_vec(self.cmds.clone(), cmd.into()));
    }

    pub fn add_file_dependency<S: Into<String>>(&mut self, file: S) {
        self.only_include_files = Some(add_to_option_vec(
            self.only_include_files.clone(),
            file.into(),
        ));
    }

    pub fn add_cache_directory<S: Into<String>>(&mut self, dir: S) {
        self.cache_directories = Some(add_to_option_vec(
            self.cache_directories.clone(),
            dir.into(),
        ));
    }

    pub fn add_path(&mut self, path: String) {
        self.paths = Some(add_to_option_vec(self.paths.clone(), path));
    }

    pub fn set_nix_archive(&mut self, archive: String) {
        self.nixpacks_archive = Some(archive);
    }
}

impl StartPhase {
    pub fn new<S: Into<String>>(cmd: S) -> Self {
        Self {
            cmd: Some(cmd.into()),
            ..Default::default()
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

    pub fn add_file_dependency<S: Into<String>>(&mut self, file: S) {
        self.only_include_files = Some(add_to_option_vec(
            self.only_include_files.clone(),
            file.into(),
        ));
    }
}

fn add_to_option_vec<T>(values: Option<Vec<T>>, v: T) -> Vec<T> {
    if let Some(mut values) = values {
        values.push(v);
        values
    } else {
        vec![v]
    }
}

fn add_multiple_to_option_vec<T: Clone>(values: Option<Vec<T>>, new_values: Vec<T>) -> Vec<T> {
    if let Some(values) = values {
        [values, new_values].concat()
    } else {
        new_values
    }
}
