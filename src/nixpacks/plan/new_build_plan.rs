use serde::{Deserialize, Serialize};

use crate::nixpacks::nix::pkg::Pkg;

use super::topological_sort::TopItem;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct GenericPhase {
    pub name: String,

    #[serde(rename = "dependsOn")]
    pub depends_on: Option<Vec<String>>,

    #[serde(rename = "nixPackages")]
    pub nix_pkgs: Option<Vec<Pkg>>,

    #[serde(rename = "nixLibraries")]
    pub nix_libraries: Option<Vec<String>>,

    #[serde(rename = "aptPackages")]
    pub apt_pkgs: Option<Vec<String>>,

    #[serde(rename = "commands")]
    pub cmds: Option<Vec<String>>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,

    #[serde(rename = "cacheDirectories")]
    pub cache_directories: Option<Vec<String>>,
}

impl TopItem for GenericPhase {
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

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct NewBuildPlan {
    // #[serde(rename = "nixpacksVersion")]
    // pub nixpacks_version: Option<String>,

    // #[serde(rename = "nixpacksArchive")]
    // pub nixpacks_archive: Option<String>,

    // #[serde(rename = "buildImage")]
    // pub build_image: String,

    // #[serde(rename = "runImage")]
    // pub run_image: Option<String>,
    pub phases: Vec<GenericPhase>,
}

impl GenericPhase {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn depends_on_phase<S: Into<String>>(&mut self, name: S) {
        self.depends_on = add_to_option_vec(self.depends_on.clone(), name.into());
    }

    pub fn add_nix_pkgs(&mut self, new_pkgs: Vec<Pkg>) {
        self.nix_pkgs = add_multiple_to_option_vec(self.nix_pkgs.clone(), new_pkgs);
    }

    pub fn add_pkgs_libs(&mut self, new_libraries: Vec<String>) {
        self.nix_libraries = add_multiple_to_option_vec(self.nix_libraries.clone(), new_libraries);
    }

    pub fn add_apt_pkgs(&mut self, new_pkgs: Vec<String>) {
        self.apt_pkgs = add_multiple_to_option_vec(self.apt_pkgs.clone(), new_pkgs);
    }

    pub fn add_cmd<S: Into<String>>(&mut self, cmd: S) {
        self.cmds = add_to_option_vec(self.cmds.clone(), cmd.into());
    }

    pub fn add_file_dependency<S: Into<String>>(&mut self, file: S) {
        self.only_include_files = add_to_option_vec(self.only_include_files.clone(), file.into());
    }

    pub fn add_cache_directory<S: Into<String>>(&mut self, dir: S) {
        self.cache_directories = add_to_option_vec(self.cache_directories.clone(), dir.into());
    }
}

impl NewBuildPlan {
    pub fn new(phases: Vec<GenericPhase>) -> Self {
        Self { phases }
    }

    pub fn add_phase(&mut self, phase: GenericPhase) {
        self.phases.push(phase);
    }
}

fn add_to_option_vec<T>(values: Option<Vec<T>>, v: T) -> Option<Vec<T>> {
    if let Some(mut values) = values {
        values.push(v);
        Some(values)
    } else {
        Some(vec![v])
    }
}

fn add_multiple_to_option_vec<T: Clone>(
    values: Option<Vec<T>>,
    new_values: Vec<T>,
) -> Option<Vec<T>> {
    if let Some(values) = values {
        Some([values, new_values].concat())
    } else {
        Some(new_values)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        nixpacks::plan::topological_sort::{self, topological_sort},
        providers::node::NODE_OVERLAY,
    };

    use super::*;

    #[test]
    fn test_adding_value_to_option_vec() {
        assert_eq!(add_to_option_vec(None, "a"), Some(vec!["a"]));
        assert_eq!(
            add_to_option_vec(Some(vec!["a", "b"]), "c"),
            Some(vec!["a", "b", "c"])
        );
    }

    #[test]
    fn test_adding_multiple_values_to_option_vec() {
        assert_eq!(
            add_multiple_to_option_vec(None, vec!["a", "b"]),
            Some(vec!["a", "b"])
        );
        assert_eq!(
            add_multiple_to_option_vec(Some(vec!["a", "b"]), vec!["c", "d"]),
            Some(vec!["a", "b", "c", "d"])
        );
    }

    #[test]
    fn test_sorting_phases() {
        let mut setup_phase = GenericPhase::new("setup");
        setup_phase.add_nix_pkgs(vec![
            Pkg::new("nodejs"),
            Pkg::new("npm-8_x").from_overlay(NODE_OVERLAY),
        ]);

        let mut install_phase = GenericPhase::new("install");
        install_phase.depends_on_phase("setup");
        install_phase.add_cmd("npm install");
        install_phase.add_cache_directory("node_modules/.cache");
        install_phase.add_cache_directory("/root/.npm");

        let mut build_phase = GenericPhase::new("build");
        build_phase.depends_on_phase("install");
        build_phase.add_cmd("npm run build");
        build_phase.add_cache_directory("node_modules/.cache");

        let mut start_phase = GenericPhase::new("start");
        start_phase.depends_on_phase("build");
        start_phase.add_cmd("npm run start");

        let plan = NewBuildPlan::new(vec![setup_phase, install_phase, build_phase, start_phase]);

        let sorted_phases = topological_sort(plan.phases)
            .unwrap()
            .into_iter()
            .map(|phase| phase.name)
            .collect::<Vec<_>>();
        assert_eq!(sorted_phases, vec!["setup", "install", "build", "start"]);
    }
}
