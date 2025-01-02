use crate::nixpacks::{
    images::{DEFAULT_BASE_IMAGE, STANDALONE_IMAGE},
    nix::{pkg::Pkg, NIXPACKS_ARCHIVE_LEGACY_OPENSSL, NIXPKGS_ARCHIVE},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

use super::utils::remove_autos_from_vec;

pub type Phases = BTreeMap<String, Phase>;

/// Holds the packages, commands, and directories needed for part of a build.
#[serde_with::skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Phase {
    pub name: Option<String>,

    #[serde(rename = "dependsOn")]
    pub depends_on: Option<Vec<String>>,

    #[serde(alias = "nixPackages")]
    pub nix_pkgs: Option<Vec<String>>,

    #[serde(alias = "nixLibraries")]
    pub nix_libs: Option<Vec<String>>,

    pub nix_overlays: Option<Vec<String>>,

    pub nixpkgs_archive: Option<String>,

    #[serde(alias = "aptPackages")]
    pub apt_pkgs: Option<Vec<String>>,

    #[serde(alias = "commands")]
    pub cmds: Option<Vec<String>>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,

    #[serde(rename = "cacheDirectories")]
    pub cache_directories: Option<Vec<String>>,

    #[serde(alias = "envPaths")]
    pub paths: Option<Vec<String>>,
}

/// Represents the final step of a container image, contains the startup command, any necessary files, and the final image that gets run by Docker.
#[serde_with::skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartPhase {
    pub cmd: Option<String>,
    pub run_image: Option<String>,
    pub only_include_files: Option<Vec<String>>,
    pub user: Option<String>,
}

impl Phase {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Returns the name of this phase.
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    /// Prefixes the name of this phase with the provided string.
    ///
    /// Used for multi-provider builds to prefix a provider's name to the name of the phases it generated.
    pub fn prefix_name(&mut self, prefix: &str) {
        self.name = Some(format!("{prefix}:{}", self.get_name()));
    }

    /// Prefixes the name of phases depended on by this phase with the provided string.
    ///
    /// Used for multi-provider builds to prefix a provider's name to the name of the dependencies in phases it generated.
    pub fn prefix_dependencies(&mut self, prefix: &str) {
        if let Some(depends_on) = &self.depends_on {
            self.depends_on = Some(
                depends_on
                    .clone()
                    .iter()
                    .map(|name| format!("{prefix}:{name}"))
                    .collect::<Vec<_>>(),
            );
        }
    }

    /// Set the name of this phase.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    /// Shortcut for creating a setup phase from a list of nix packages.
    pub fn setup(pkgs: Option<Vec<Pkg>>) -> Self {
        Self {
            nix_pkgs: pkgs
                .clone()
                .map(|pkgs| pkgs.iter().map(Pkg::to_nix_string).collect()),
            nix_overlays: pkgs
                .map(|pkgs| pkgs.iter().filter_map(|pkg| pkg.overlay.clone()).collect()),
            name: Some("setup".to_string()),
            ..Default::default()
        }
    }

    /// Shortcut for creating an install phase from a command
    pub fn install(cmd: Option<String>) -> Self {
        Self {
            name: Some("install".to_string()),
            cmds: cmd.map(|cmd| vec![cmd]),
            depends_on: Some(vec!["setup".to_string()]),
            ..Default::default()
        }
    }

    /// Shortcut for creating a build phase from a command
    pub fn build(cmd: Option<String>) -> Self {
        Self {
            name: Some("build".to_string()),
            cmds: cmd.map(|cmd| vec![cmd]),
            depends_on: Some(vec!["install".to_string()]),
            ..Default::default()
        }
    }

    /// Whether or not the phase uses Nix in any way
    pub fn uses_nix(&self) -> bool {
        !self.nix_pkgs.clone().unwrap_or_default().is_empty()
            || !self.nix_libs.clone().unwrap_or_default().is_empty()
    }

    /// Whether or not the phase runs any docker commands
    pub fn runs_docker_commands(&self) -> bool {
        !self.cmds.clone().unwrap_or_default().is_empty()
            || !self.paths.clone().unwrap_or_default().is_empty()
    }

    /// Add a phase name as a dependency of this phase.
    pub fn depends_on_phase<S: Into<String>>(&mut self, name: S) {
        self.depends_on = Some(add_to_option_vec(self.depends_on.clone(), name.into()));
    }

    /// Add a collection of packages to install with Nix in this phase.
    pub fn add_nix_pkgs(&mut self, new_pkgs: &[Pkg]) {
        self.nix_overlays = Some(add_multiple_to_option_vec(
            self.nix_overlays.clone(),
            new_pkgs
                .iter()
                .filter_map(|pkg| pkg.overlay.clone())
                .collect::<Vec<_>>(),
        ));
        self.nix_pkgs = Some(add_multiple_to_option_vec(
            self.nix_pkgs.clone(),
            new_pkgs.iter().map(Pkg::to_nix_string).collect(),
        ));
    }

    /// Add a collection of libraries to install with Nix in this phase.
    pub fn add_pkgs_libs(&mut self, new_libraries: Vec<String>) {
        self.nix_libs = Some(add_multiple_to_option_vec(
            self.nix_libs.clone(),
            new_libraries,
        ));
    }

    /// Add a collection of packages to install with apt in this phase.
    pub fn add_apt_pkgs(&mut self, new_pkgs: Vec<String>) {
        self.apt_pkgs = Some(add_multiple_to_option_vec(self.apt_pkgs.clone(), new_pkgs));
    }

    /// Add a command to execute in this phase.
    pub fn add_cmd<S: Into<String>>(&mut self, cmd: S) {
        self.cmds = Some(add_to_option_vec(self.cmds.clone(), cmd.into()));
    }

    /// Add a file to the list of files copied into the container in this phase.
    pub fn add_file_dependency<S: Into<String>>(&mut self, file: S) {
        self.only_include_files = Some(add_to_option_vec(
            self.only_include_files.clone(),
            file.into(),
        ));
    }

    /// Add a directory in which language-specific packages get installed.
    pub fn add_cache_directory<S: Into<String>>(&mut self, dir: S) {
        let mut new_directories = prevent_duplicates_vec(add_to_option_vec(
            self.cache_directories.clone(),
            dir.into(),
        ));
        new_directories.sort();
        self.cache_directories = Some(new_directories);
    }

    /// Add the given path to a list of paths that should be present in the built image.
    pub fn add_path(&mut self, path: String) {
        self.paths = Some(add_to_option_vec(self.paths.clone(), path));
    }

    /// Set the nixpkgs revision used by this phase.
    pub fn set_nix_archive(&mut self, archive: String) {
        self.nixpkgs_archive = Some(archive);
    }

    /// Store the phase dependencies for later reproducibility.
    pub fn pin(&mut self, use_legacy_openssl: bool) {
        if self.uses_nix() && self.nixpkgs_archive.is_none() {
            self.nixpkgs_archive = if use_legacy_openssl {
                Some(NIXPACKS_ARCHIVE_LEGACY_OPENSSL.to_string())
            } else {
                Some(NIXPKGS_ARCHIVE.to_string())
            }
        }

        self.cmds = pin_option_vec(self.cmds.as_ref());
        self.depends_on = pin_option_vec(self.depends_on.as_ref());
        self.nix_pkgs = pin_option_vec(self.nix_pkgs.as_ref());
        self.nix_libs = pin_option_vec(self.nix_libs.as_ref());
        self.apt_pkgs = pin_option_vec(self.apt_pkgs.as_ref());
        self.nix_overlays = pin_option_vec(self.nix_overlays.as_ref());
        self.only_include_files = pin_option_vec(self.only_include_files.as_ref());
        self.cache_directories = pin_option_vec(self.cache_directories.as_ref());
        self.paths = pin_option_vec(self.paths.as_ref());
    }
}

impl StartPhase {
    pub fn new<S: Into<String>>(cmd: S) -> Self {
        Self {
            cmd: Some(cmd.into()),
            ..Default::default()
        }
    }

    /// Set the container image in which to run the StartPhase.
    pub fn run_in_image(&mut self, image_name: String) {
        self.run_image = Some(image_name);
    }

    /// Run the StartPhase in the default base image.
    pub fn run_in_default_image(&mut self) {
        self.run_image = Some(DEFAULT_BASE_IMAGE.to_string());
    }

    /// Run the StartPhase in a generic image.
    pub fn run_in_slim_image(&mut self) {
        self.run_image = Some(STANDALONE_IMAGE.to_string());
    }

    /// Add a file to the set of files to copy into the container image.
    pub fn add_file_dependency<S: Into<String>>(&mut self, file: S) {
        self.only_include_files = Some(add_to_option_vec(
            self.only_include_files.clone(),
            file.into(),
        ));
    }

    /// Store the list of files to include in this phase for later reproducibility.
    pub fn pin(&mut self) {
        self.only_include_files = pin_option_vec(self.only_include_files.as_ref());
    }
}

/// Store the list of options for this phase for later reproducibility.
fn pin_option_vec(vec: Option<&Vec<String>>) -> Option<Vec<String>> {
    vec.map(|vec| remove_autos_from_vec(vec.clone()))
}

/// Add an option to the vector of options for the phase.
fn add_to_option_vec<T>(values: Option<Vec<T>>, v: T) -> Vec<T> {
    if let Some(mut values) = values {
        values.push(v);
        values
    } else {
        vec![v]
    }
}

/// Add multiple options to the vector of options for the phase.
fn add_multiple_to_option_vec<T: Clone>(values: Option<Vec<T>>, new_values: Vec<T>) -> Vec<T> {
    if let Some(values) = values {
        [values, new_values].concat()
    } else {
        new_values
    }
}

/// Ensure that files aren't copied into the image twice.
#[allow(clippy::needless_pass_by_value)]
fn prevent_duplicates_vec<T: Clone + Eq + Hash>(values: Vec<T>) -> Vec<T> {
    let set: HashSet<T> = values.iter().cloned().collect::<HashSet<_>>();
    set.into_iter().collect()
}
