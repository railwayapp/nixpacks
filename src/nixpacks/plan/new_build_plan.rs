use super::topological_sort::{topological_sort, TopItem};
use crate::nixpacks::{
    app::StaticAssets,
    environment::EnvironmentVariables,
    images::{DEBIAN_SLIM_IMAGE, DEFAULT_BASE_IMAGE},
    nix::pkg::Pkg,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct NewPhase {
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
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct NewStartPhase {
    pub cmd: Option<String>,

    #[serde(rename = "runImage")]
    pub run_image: Option<String>,

    #[serde(rename = "onlyIncludeFiles")]
    pub only_include_files: Option<Vec<String>>,
}

impl TopItem for NewPhase {
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
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NewBuildPlan {
    // #[serde(rename = "nixpacksVersion")]
    // pub nixpacks_version: Option<String>,

    // #[serde(rename = "nixpacksArchive")]
    // pub nixpacks_archive: Option<String>,

    // #[serde(rename = "buildImage")]
    // pub build_image: String,
    pub variables: Option<EnvironmentVariables>,

    #[serde(rename = "staticAssets")]
    pub static_assets: Option<StaticAssets>,

    pub phases: Vec<NewPhase>,

    #[serde(rename = "startPhase")]
    pub start_phase: Option<NewStartPhase>,
}

impl NewPhase {
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
    pub fn new(phases: Vec<NewPhase>) -> Self {
        Self {
            phases,
            ..Default::default()
        }
    }

    pub fn add_phase(&mut self, phase: NewPhase) {
        self.phases.push(phase);
    }

    pub fn add_start_phase(&mut self, start_phase: NewStartPhase) {
        self.start_phase = Some(start_phase);
    }

    pub fn set_variables(&mut self, variables: EnvironmentVariables) {
        self.variables = Some(variables);
    }

    pub fn get_sorted_phases(&self) -> Result<Vec<NewPhase>> {
        topological_sort(self.phases.clone())
    }
}

impl NewStartPhase {
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
        self.only_include_files = add_to_option_vec(self.only_include_files.clone(), file.into());
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
    use super::*;

    use crate::providers::node::NODE_OVERLAY;

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
        let mut setup_phase = NewPhase::new("setup");
        setup_phase.add_nix_pkgs(vec![
            Pkg::new("nodejs"),
            Pkg::new("npm-8_x").from_overlay(NODE_OVERLAY),
        ]);

        let mut install_phase = NewPhase::new("install");
        install_phase.depends_on_phase("setup");
        install_phase.add_cmd("npm install");
        install_phase.add_cache_directory("node_modules/.cache");
        install_phase.add_cache_directory("/root/.npm");

        let mut build_phase = NewPhase::new("build");
        build_phase.depends_on_phase("install");
        build_phase.add_cmd("npm run build");
        build_phase.add_cache_directory("node_modules/.cache");

        let mut start_phase = NewPhase::new("start");
        start_phase.depends_on_phase("build");
        start_phase.add_cmd("npm run start");

        let plan = NewBuildPlan::new(vec![install_phase, build_phase, setup_phase]);

        let sorted_phases = plan
            .get_sorted_phases()
            .unwrap()
            .into_iter()
            .map(|phase| phase.name)
            .collect::<Vec<_>>();
        assert_eq!(sorted_phases, vec!["setup", "install", "build", "start"]);
    }
}
