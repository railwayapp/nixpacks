use crate::nixpacks::{
    images::{DEBIAN_SLIM_IMAGE, DEFAULT_BASE_IMAGE},
    nix::{pkg::Pkg, NIXPKGS_ARCHIVE},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Phases = BTreeMap<String, Phase>;

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

#[serde_with::skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartPhase {
    pub cmd: Option<String>,
    pub run_image: Option<String>,
    pub only_include_files: Option<Vec<String>>,
}

impl Phase {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    pub fn prefix_name(&mut self, prefix: &str) {
        self.name = Some(format!("{}:{}", prefix, self.get_name()));
    }

    pub fn prefix_dependencies(&mut self, prefix: &str) {
        if let Some(depends_on) = &self.depends_on {
            self.depends_on = Some(
                depends_on
                    .clone()
                    .iter()
                    .map(|name| format!("{}:{}", prefix, name))
                    .collect::<Vec<_>>(),
            );
        }
    }

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

    pub fn uses_nix(&self) -> bool {
        !self.nix_pkgs.clone().unwrap_or_default().is_empty()
            || !self.nix_libs.clone().unwrap_or_default().is_empty()
    }

    pub fn depends_on_phase<S: Into<String>>(&mut self, name: S) {
        self.depends_on = Some(add_to_option_vec(self.depends_on.clone(), name.into()));
    }

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

    pub fn add_pkgs_libs(&mut self, new_libraries: Vec<String>) {
        self.nix_libs = Some(add_multiple_to_option_vec(
            self.nix_libs.clone(),
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
        self.nixpkgs_archive = Some(archive);
    }

    pub fn pin(&mut self) {
        if self.uses_nix() && self.nixpkgs_archive.is_none() {
            self.nixpkgs_archive = Some(NIXPKGS_ARCHIVE.to_string());
        }

        self.cmds = pin_option_vec(&self.cmds);
        self.depends_on = pin_option_vec(&self.depends_on);
        self.nix_pkgs = pin_option_vec(&self.nix_pkgs);
        self.nix_libs = pin_option_vec(&self.nix_libs);
        self.apt_pkgs = pin_option_vec(&self.apt_pkgs);
        self.nix_overlays = pin_option_vec(&self.nix_overlays);
        self.only_include_files = pin_option_vec(&self.only_include_files);
        self.cache_directories = pin_option_vec(&self.cache_directories);
        self.paths = pin_option_vec(&self.paths);
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

    pub fn pin(&mut self) {
        self.only_include_files = pin_option_vec(&self.only_include_files);
    }
}

fn pin_option_vec(vec: &Option<Vec<String>>) -> Option<Vec<String>> {
    if let Some(vec) = vec {
        Some(remove_autos_from_vec(vec.clone()))
    } else {
        vec.clone()
    }
}

/// Removes all the `"..."`'s or `"@auto"`'s from the `original`
fn remove_autos_from_vec(original: Vec<String>) -> Vec<String> {
    original
        .into_iter()
        .filter(|x| x != "@auto" && x != "...")
        .collect::<Vec<_>>()
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

#[cfg(test)]
mod test {
    use super::*;

    fn vs(v: Vec<&str>) -> Vec<String> {
        v.into_iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    #[test]
    fn test_remove_autos_from_vec() {
        assert_eq!(
            vs(vec!["a", "b", "c"]),
            remove_autos_from_vec(vs(vec!["a", "b", "c"]))
        );
        assert_eq!(
            vs(vec!["a", "c"]),
            remove_autos_from_vec(vs(vec!["a", "...", "c"]))
        );
        assert_eq!(
            vs(vec!["a", "c"]),
            remove_autos_from_vec(vs(vec!["@auto", "a", "...", "c", "@auto"]))
        );
    }
}
