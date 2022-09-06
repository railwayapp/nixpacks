use core::fmt;
use std::{collections::BTreeMap, convert::identity};

use self::{
    merge::Mergeable,
    phase::{Phase, Phases, StartPhase},
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

// pub mod config;
pub mod generator;
pub mod merge;
pub mod phase;
pub mod pretty_print;
mod topological_sort;

pub trait PlanGenerator {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan>;
}

#[serde_with::skip_serializing_none]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildPlan {
    pub providers: Option<Vec<String>>,

    #[serde(rename = "buildImage")]
    pub build_image: Option<String>,

    pub variables: Option<EnvironmentVariables>,

    #[serde(rename = "staticAssets")]
    pub static_assets: Option<StaticAssets>,

    pub phases: Option<Phases>,

    #[serde(rename = "start")]
    pub start_phase: Option<StartPhase>,
}

impl BuildPlan {
    pub fn new(phases: Vec<Phase>, start_phase: Option<StartPhase>) -> Self {
        Self {
            phases: Some(phases.iter().map(|p| (p.get_name(), p.clone())).collect()),
            start_phase,
            build_image: Some(DEFAULT_BASE_IMAGE.to_string()),
            ..Default::default()
        }
    }

    pub fn from_toml<S: Into<String>>(toml: S) -> Result<Self> {
        let mut plan: BuildPlan = toml::from_str(&toml.into())?;
        plan.resolve_phase_names();
        Ok(plan)
    }

    pub fn add_phase(&mut self, phase: Phase) {
        let phases = self.phases.get_or_insert(BTreeMap::default());
        phases.insert(phase.get_name(), phase);
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
        match self.phases {
            Some(ref phases) => phases.get(name),
            None => None,
        }
    }

    pub fn get_phase_mut(&mut self, name: &str) -> Option<&mut Phase> {
        self.phases.get_or_insert(BTreeMap::default()).get_mut(name)
    }

    pub fn remove_phase(&mut self, name: &str) -> Option<Phase> {
        self.phases.get_or_insert(BTreeMap::default()).remove(name)
    }

    pub fn get_sorted_phases(&self) -> Result<Vec<Phase>> {
        let phases_with_names = self
            .phases
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|(name, phase)| (name.clone(), phase.clone()))
            .collect();

        let res = topological_sort::<(String, Phase)>(phases_with_names)?
            .iter()
            .map(|(_, phase)| phase.clone())
            .collect();

        Ok(res)
    }

    pub fn get_phases_with_dependencies(&self, phase_name: &str) -> Phases {
        let p = self.get_phase(phase_name);

        let mut phases = Phases::new();
        let mut deps: Vec<String> = Vec::new();

        if let Some(p) = p {
            phases.insert(phase_name.to_string(), p.clone());
            deps.append(&mut p.clone().depends_on.unwrap_or_default());

            while !deps.is_empty() {
                let dep = deps.pop().unwrap();
                let p = self.get_phase(&dep);
                if let Some(p) = p {
                    phases.insert(dep, p.clone());
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
        for (name, mut phase) in phases {
            phase.prefix_name(prefix);
            self.add_phase(phase);
        }

        format!("{}:{}", prefix, phase_name)
    }

    pub fn add_dependency_between_phases(&mut self, dependant: &str, dependency: &str) {
        if let Some(p) = self.get_phase_mut(dependant) {
            p.depends_on_phase(dependency);
        }
    }

    pub fn resolve_phase_names(&mut self) {
        let mut phases = self.phases.get_or_insert(BTreeMap::default());
        for (name, phase) in phases.iter_mut() {
            phase.set_name(name);
        }
    }

    pub fn from_environment(env: &Environment) -> Self {
        let mut phases: Vec<Phase> = Vec::new();

        // Setup
        let mut setup = Phase::setup(None);
        let mut uses_setup = false;

        if let Some(pkg_string) = env.get_config_variable("PKGS") {
            let mut pkgs = pkg_string
                .split(' ')
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
            pkgs.push("...".to_string());
            setup.nix_pkgs = Some(pkgs);
            uses_setup = true;
        }
        if let Some(apt_string) = env.get_config_variable("APT_PKGS") {
            let mut apts = apt_string
                .split(' ')
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
            apts.push("...".to_string());
            setup.apt_pkgs = Some(apts);
            uses_setup = true;
        }
        if let Some(nix_lib_string) = env.get_config_variable("LIBS") {
            let mut libs = nix_lib_string
                .split(' ')
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
            libs.push("...".to_string());
            setup.nix_libs = Some(libs);
            uses_setup = true;
        }

        if uses_setup {
            phases.push(setup);
        }

        // Install
        if let Some(cmd_string) = env.get_config_variable("INSTALL_CMD") {
            let install = Phase::install(Some(cmd_string));
            phases.push(install);
        }

        // Build
        if let Some(cmd_string) = env.get_config_variable("BUILD_CMD") {
            let build = Phase::build(Some(cmd_string));
            phases.push(build);
        }

        // Start
        let start = env.get_config_variable("START_CMD").map(StartPhase::new);

        BuildPlan::new(phases, start)
    }

    pub fn pin(&mut self) {
        self.resolve_phase_names();
        let phases = self.phases.get_or_insert(Phases::default());
        for (_, phase) in phases.iter_mut() {
            phase.pin();
        }
    }

    pub fn merge_plans(plans: Vec<BuildPlan>) -> BuildPlan {
        plans.iter().fold(BuildPlan::default(), |acc, plan| {
            BuildPlan::merge(&acc, plan)
        })
    }

    // Create a new build plan by applying the given configuration
    // pub fn apply_config(plan: &BuildPlan, config: &NixpacksConfig) -> BuildPlan {
    //     let mut new_plan = plan.clone();

    //     for (name, phase_config) in config.phases.clone().unwrap_or_default() {
    //         let phase = new_plan.remove_phase(name.as_str());
    //         let mut phase = phase.unwrap_or_else(|| {
    //             let mut phase = Phase::new(name.clone());
    //             if name == "install" {
    //                 phase.depends_on_phase("setup");
    //             } else if name == "build" {
    //                 phase.depends_on_phase("install");
    //             };

    //             phase
    //         });

    //         if let Some(cmds) = phase_config.cmds {
    //             phase.cmds = Some(replace_auto_vec(
    //                 cmds,
    //                 &phase.cmds.clone().unwrap_or_default(),
    //                 identity,
    //             ));
    //         }

    //         if let Some(nix_pkgs) = phase_config.nix_pkgs {
    //             phase.nix_pkgs = Some(replace_auto_vec(
    //                 nix_pkgs,
    //                 &phase.nix_pkgs.clone().unwrap_or_default(),
    //                 identity,
    //             ));
    //         }

    //         if let Some(apt_pkgs) = phase_config.apt_pkgs {
    //             phase.apt_pkgs = Some(replace_auto_vec(
    //                 apt_pkgs,
    //                 &phase.apt_pkgs.clone().unwrap_or_default(),
    //                 identity,
    //             ));
    //         }

    //         if let Some(nix_libs) = phase_config.nix_libs {
    //             phase.nix_libs = Some(replace_auto_vec(
    //                 nix_libs,
    //                 &phase.nix_libs.clone().unwrap_or_default(),
    //                 identity,
    //             ));
    //         }

    //         if let Some(depends_on) = phase_config.depends_on {
    //             phase.depends_on = Some(replace_auto_vec(
    //                 depends_on,
    //                 &phase.depends_on.clone().unwrap_or_default(),
    //                 identity,
    //             ));
    //         }

    //         new_plan.add_phase(phase);
    //     }

    //     let mut start_phase = new_plan.start_phase.clone().unwrap_or_default();
    //     start_phase.cmd = config.start_cmd.clone().or(start_phase.cmd);
    //     new_plan.set_start_phase(start_phase);

    //     new_plan
    // }
}

impl topological_sort::TopItem for (String, Phase) {
    fn get_name(&self) -> String {
        self.0.clone()
    }

    fn get_dependencies(&self) -> &[String] {
        match &self.1.depends_on {
            Some(depends_on) => depends_on.as_slice(),
            None => &[],
        }
    }
}

impl Default for BuildPlan {
    fn default() -> Self {
        Self {
            providers: None,
            build_image: Some(DEFAULT_BASE_IMAGE.to_string()),
            phases: Some(Phases::default()),
            start_phase: None,
            variables: Some(EnvironmentVariables::default()),
            static_assets: Some(StaticAssets::default()),
        }
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

        // let phases = topological_sort(plan.get_phases_with_dependencies("build")).unwrap();
        let build_phase = plan.get_phases_with_dependencies("build");
        let phases = build_phase.values().collect::<Vec<_>>();

        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].get_name(), "setup");
        assert_eq!(phases[1].get_name(), "install");
        assert_eq!(phases[2].get_name(), "build");
    }

    // #[test]
    // fn test_apply_config_to_plan() {
    //     let mut setup = Phase::new("setup");
    //     setup.add_nix_pkgs(vec![Pkg::new("wget")]);
    //     let mut install = Phase::new("install");
    //     install.depends_on_phase("setup");
    //     let mut build = Phase::new("build");
    //     build.depends_on_phase("install");
    //     let mut another = Phase::new("another");
    //     another.depends_on_phase("setup");
    //     let plan = BuildPlan::new(vec![setup, install, build, another], None);

    //     let plan = BuildPlan::apply_config(
    //         &plan,
    //         &NixpacksConfig {
    //             phases: Some(BTreeMap::from([
    //                 (
    //                     "setup".to_string(),
    //                     PhaseConfig {
    //                         nix_pkgs: Some(vec!["cowsay".to_string()]),
    //                         ..Default::default()
    //                     },
    //                 ),
    //                 (
    //                     "build".to_string(),
    //                     PhaseConfig {
    //                         cmds: Some(vec!["yarn run optimize-assets".to_string()]),
    //                         ..Default::default()
    //                     },
    //                 ),
    //             ])),
    //             ..Default::default()
    //         },
    //     );

    //     println!("{}", serde_json::to_string_pretty(&plan).unwrap());

    //     assert_eq!(
    //         vec!["wget".to_string(), "cowsay".to_string()],
    //         plan.get_phase("setup").unwrap().nix_pkgs.clone().unwrap()
    //     );
    //     assert_eq!(
    //         "yarn run optimize-assets",
    //         plan.get_phase("build").unwrap().cmds.clone().unwrap()[0]
    //     );
    // }
}
