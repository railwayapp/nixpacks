use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    images::{DEBIAN_SLIM_IMAGE, DEFAULT_BASE_IMAGE},
    nix::pkg::Pkg,
};

use super::{
    legacy_phase::{LegacyBuildPhase, LegacyInstallPhase, LegacySetupPhase, LegacyStartPhase},
    topological_sort::TopItem,
};

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

// Legacy intos

impl From<LegacySetupPhase> for Phase {
    fn from(setup_phase: LegacySetupPhase) -> Self {
        Phase {
            name: "setup".to_string(),
            nix_pkgs: Some(setup_phase.pkgs),
            nix_libraries: setup_phase.libraries,
            nixpacks_archive: setup_phase.archive,
            apt_pkgs: setup_phase.apt_pkgs,
            cmds: setup_phase.cmds,
            only_include_files: setup_phase.only_include_files,
            ..Default::default()
        }
    }
}

impl From<LegacyInstallPhase> for Phase {
    fn from(install_phase: LegacyInstallPhase) -> Self {
        let mut i = Phase {
            name: "install".to_string(),
            cmds: install_phase.cmds,
            only_include_files: install_phase.only_include_files,
            cache_directories: install_phase.cache_directories,
            paths: install_phase.paths,
            ..Default::default()
        };

        i.depends_on_phase("setup");
        i
    }
}

impl From<LegacyBuildPhase> for Phase {
    fn from(build_phase: LegacyBuildPhase) -> Self {
        let mut p = Phase {
            name: "build".to_string(),
            cmds: build_phase.cmds,
            only_include_files: build_phase.only_include_files,
            cache_directories: build_phase.cache_directories,
            ..Default::default()
        };

        p.depends_on_phase("install");
        p
    }
}

impl From<LegacyStartPhase> for StartPhase {
    fn from(start_phase: LegacyStartPhase) -> Self {
        StartPhase {
            run_image: start_phase.run_image,
            cmd: start_phase.cmd,
            ..Default::default()
        }
    }
}
