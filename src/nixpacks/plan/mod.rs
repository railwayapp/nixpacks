use self::{
    config::GeneratePlanConfig,
    generator::NIXPKGS_ARCHIVE,
    phase::{Phase, StartPhase},
    topological_sort::topological_sort,
};
use super::{images::DEFAULT_BASE_IMAGE, NIX_PACKS_VERSION};
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
};

use anyhow::Result;

use serde::{Deserialize, Serialize};

pub mod config;
pub mod generator;
pub mod phase;
pub mod pretty_print;
mod topological_sort;

pub trait PlanGenerator {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan>;
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildPlan {
    #[serde(rename = "nixpacksVersion")]
    nixpacks_version: Option<String>,

    #[serde(rename = "buildImage")]
    pub build_image: String,

    pub variables: Option<EnvironmentVariables>,

    #[serde(rename = "staticAssets")]
    pub static_assets: Option<StaticAssets>,

    pub phases: Vec<Phase>,

    #[serde(rename = "startPhase")]
    pub start_phase: Option<StartPhase>,
}

impl BuildPlan {
    pub fn new(phases: Vec<Phase>, start_phase: Option<StartPhase>) -> Self {
        Self {
            nixpacks_version: Some(NIX_PACKS_VERSION.to_string()),
            phases,
            start_phase,
            build_image: DEFAULT_BASE_IMAGE.to_string(),
            ..Default::default()
        }
    }

    pub fn add_phase(&mut self, phase: Phase) {
        self.phases.push(phase);
    }

    pub fn set_start_phase(&mut self, start_phase: StartPhase) {
        self.start_phase = Some(start_phase);
    }

    pub fn add_variables(&mut self, variables: EnvironmentVariables) {
        match self.variables.as_mut() {
            Some(vars) => {
                for (key, value) in &variables {
                    vars.insert(key.to_string(), value.to_string());
                }
            }
            None => {
                self.variables = Some(variables);
            }
        }
    }

    pub fn add_static_assets(&mut self, static_assets: StaticAssets) {
        match self.static_assets.as_mut() {
            Some(assets) => {
                for (key, value) in &static_assets {
                    assets.insert(key.to_string(), value.to_string());
                }
            }
            None => {
                self.static_assets = Some(static_assets);
            }
        }
    }

    pub fn get_phase(&self, name: &str) -> Option<&Phase> {
        self.phases.iter().find(|phase| phase.name == name)
    }

    pub fn get_phase_mut(&mut self, name: &str) -> Option<&mut Phase> {
        self.phases.iter_mut().find(|phase| phase.name == name)
    }

    pub fn remove_phase(&mut self, name: &str) -> Option<Phase> {
        let index = self.phases.iter().position(|phase| phase.name == name);
        if let Some(index) = index {
            let phase = self.phases.swap_remove(index);
            Some(phase)
        } else {
            None
        }
    }

    pub fn get_sorted_phases(&self) -> Result<Vec<Phase>> {
        topological_sort(self.phases.clone())
    }

    pub fn get_phases_with_dependencies(&self, phase_name: &str) -> Vec<Phase> {
        let p = self.get_phase(phase_name);

        let mut phases = Vec::new();
        let mut deps: Vec<String> = Vec::new();

        if let Some(p) = p {
            phases.push(p.clone());
            deps.append(&mut p.clone().depends_on.unwrap_or_default());

            while !deps.is_empty() {
                let dep = deps.pop().unwrap();
                let p = self.get_phase(&dep);
                if let Some(p) = p {
                    phases.push(p.clone());
                    deps.append(&mut p.clone().depends_on.unwrap_or_default());
                }
            }
        }

        phases
    }

    pub fn add_phases_from_another_plan(
        &mut self,
        plan: &BuildPlan,
        prefix: &str,
        phase_name: &str,
    ) -> String {
        let phases = plan.get_phases_with_dependencies(phase_name);
        for mut phase in phases {
            phase.prefix_name(prefix);
            self.add_phase(phase.clone());
        }

        format!("{}:{}", prefix, phase_name)
    }

    pub fn add_dependency_between_phases(&mut self, dependant: &str, dependency: &str) {
        if let Some(p) = self.get_phase_mut(dependant) {
            p.depends_on_phase(dependency);
        }
    }

    /// Create a new build plan by applying the given configuration
    pub fn apply_config(plan: &BuildPlan, config: &GeneratePlanConfig) -> BuildPlan {
        let mut new_plan = plan.clone();

        // Setup config
        let mut setup = new_plan
            .remove_phase("setup")
            .unwrap_or_else(|| Phase::setup(None));

        // Append the packages and libraries together
        setup.apt_pkgs = none_if_empty(
            [
                config.custom_apt_pkgs.clone(),
                setup.apt_pkgs.unwrap_or_default(),
            ]
            .concat(),
        );
        setup.nix_pkgs = none_if_empty(
            [
                config.custom_pkgs.clone(),
                setup.nix_pkgs.unwrap_or_default(),
            ]
            .concat(),
        );
        setup.nix_libraries = none_if_empty(
            [
                config.custom_libs.clone(),
                setup.nix_libraries.unwrap_or_default(),
            ]
            .concat(),
        );
        setup.nixpacks_archive = setup.nixpacks_archive.or_else(|| {
            if config.pin_pkgs {
                Some(NIXPKGS_ARCHIVE.to_string())
            } else {
                None
            }
        });
        new_plan.add_phase(setup);

        // Install config
        let mut install = new_plan
            .remove_phase("install")
            .unwrap_or_else(|| Phase::install(None));
        install.cmds = config.custom_install_cmd.clone().or(install.cmds);
        new_plan.add_phase(install);

        // Build config
        let mut build = new_plan
            .remove_phase("build")
            .unwrap_or_else(|| Phase::build(None));
        build.cmds = config.custom_build_cmd.clone().or(build.cmds);
        new_plan.add_phase(build);

        // Start config
        let mut start = new_plan.start_phase.clone().unwrap_or_default();
        start.cmd = config.custom_start_cmd.clone().or(start.cmd);
        new_plan.start_phase = Some(start);

        new_plan
    }
}

impl Default for BuildPlan {
    fn default() -> Self {
        Self {
            nixpacks_version: Some(NIX_PACKS_VERSION.to_string()),
            build_image: DEFAULT_BASE_IMAGE.to_string(),
            phases: vec![],
            start_phase: None,
            variables: None,
            static_assets: None,
        }
    }
}

fn none_if_empty<T>(value: Vec<T>) -> Option<Vec<T>> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_phases_with_dependencies() {
        let setup = Phase::new("setup");

        let mut install = Phase::new("install");
        install.depends_on_phase("setup");

        let mut build = Phase::new("build");
        build.depends_on_phase("install");

        let mut another = Phase::new("another");
        another.depends_on_phase("setup");

        let plan = BuildPlan::new(vec![setup, install, build, another], None);

        let phases = topological_sort(plan.get_phases_with_dependencies("build")).unwrap();

        println!("{:?}", phases);

        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].name, "setup");
        assert_eq!(phases[1].name, "install");
        assert_eq!(phases[2].name, "build");
    }
}
