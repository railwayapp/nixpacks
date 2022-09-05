use core::fmt;
use std::convert::identity;

use self::{
    config::NixpacksConfig,
    phase::{Phase, StartPhase},
    topological_sort::topological_sort,
};
use super::{images::DEFAULT_BASE_IMAGE, NIX_PACKS_VERSION};
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
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
    pub fn apply_config(plan: &BuildPlan, config: &NixpacksConfig) -> BuildPlan {
        let mut new_plan = plan.clone();

        for (name, phase_config) in config.phases.clone().unwrap_or_default() {
            let phase = new_plan.remove_phase(name.as_str());
            let mut phase = phase.unwrap_or_else(|| {
                let mut phase = Phase::new(name.clone());
                if name == "install" {
                    phase.depends_on_phase("setup");
                } else if name == "build" {
                    phase.depends_on_phase("install");
                };

                phase
            });

            if let Some(cmds) = phase_config.cmds {
                phase.cmds = Some(replace_auto_vec(
                    cmds,
                    &phase.cmds.clone().unwrap_or_default(),
                    identity,
                ));
            }

            if let Some(nix_pkgs) = phase_config.nix_pkgs {
                phase.nix_pkgs = Some(replace_auto_vec(
                    nix_pkgs,
                    &phase.nix_pkgs.clone().unwrap_or_default(),
                    identity,
                ));
            }

            if let Some(apt_pkgs) = phase_config.apt_pkgs {
                phase.apt_pkgs = Some(replace_auto_vec(
                    apt_pkgs,
                    &phase.apt_pkgs.clone().unwrap_or_default(),
                    identity,
                ));
            }

            if let Some(nix_libs) = phase_config.nix_libs {
                phase.nix_libs = Some(replace_auto_vec(
                    nix_libs,
                    &phase.nix_libs.clone().unwrap_or_default(),
                    identity,
                ));
            }

            if let Some(depends_on) = phase_config.depends_on {
                phase.depends_on = Some(replace_auto_vec(
                    depends_on,
                    &phase.depends_on.clone().unwrap_or_default(),
                    identity,
                ));
            }

            new_plan.add_phase(phase);
        }

        let mut start_phase = new_plan.start_phase.clone().unwrap_or_default();
        start_phase.cmd = config.start_cmd.clone().or(start_phase.cmd);
        new_plan.set_start_phase(start_phase);

        new_plan
    }
}

fn replace_auto_vec<T>(arr: Vec<T>, auto: &[T], selector: fn(T) -> String) -> Vec<T>
where
    T: Clone + fmt::Debug,
{
    arr.into_iter()
        .map(|x| vec![x])
        .flat_map(|pkgs| {
            let v = selector(pkgs[0].clone());
            if v == "@auto" || v == "..." {
                auto.clone().into()
            } else {
                pkgs
            }
        })
        .collect::<Vec<_>>()
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

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::{config::PhaseConfig, *};

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

        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].name, "setup");
        assert_eq!(phases[1].name, "install");
        assert_eq!(phases[2].name, "build");
    }

    #[test]
    fn test_apply_config_to_plan() {
        let mut setup = Phase::new("setup");
        setup.add_nix_pkgs(vec![Pkg::new("wget")]);
        let mut install = Phase::new("install");
        install.depends_on_phase("setup");
        let mut build = Phase::new("build");
        build.depends_on_phase("install");
        let mut another = Phase::new("another");
        another.depends_on_phase("setup");
        let plan = BuildPlan::new(vec![setup, install, build, another], None);

        let plan = BuildPlan::apply_config(
            &plan,
            &NixpacksConfig {
                phases: Some(BTreeMap::from([
                    (
                        "setup".to_string(),
                        PhaseConfig {
                            nix_pkgs: Some(vec!["cowsay".to_string()]),
                            ..Default::default()
                        },
                    ),
                    (
                        "build".to_string(),
                        PhaseConfig {
                            cmds: Some(vec!["yarn run optimize-assets".to_string()]),
                            ..Default::default()
                        },
                    ),
                ])),
                ..Default::default()
            },
        );

        println!("{}", serde_json::to_string_pretty(&plan).unwrap());

        assert_eq!(
            vec!["wget".to_string(), "cowsay".to_string()],
            plan.get_phase("setup").unwrap().nix_pkgs.clone().unwrap()
        );
        assert_eq!(
            "yarn run optimize-assets",
            plan.get_phase("build").unwrap().cmds.clone().unwrap()[0]
        );
    }
}
