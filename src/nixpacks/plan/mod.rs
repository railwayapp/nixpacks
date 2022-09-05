use core::fmt;
use std::{collections::BTreeMap, convert::identity};

use self::{
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
    #[serde(rename = "buildImage")]
    pub build_image: Option<String>,

    pub variables: Option<EnvironmentVariables>,

    #[serde(rename = "staticAssets")]
    pub static_assets: Option<StaticAssets>,

    pub phases: Option<Phases>,

    #[serde(rename = "start")]
    pub start_phase: Option<StartPhase>,
}

pub trait Mergeable {
    fn merge(c1: &Self, c2: &Self) -> Self;
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

impl Mergeable for BuildPlan {
    fn merge(c1: &BuildPlan, c2: &BuildPlan) -> BuildPlan {
        // println!("\n\n=== MERGING ===\n");
        // println!("[c1]:\n{}\n\n", c1.get_build_string().unwrap());

        // println!(
        //     "[c2]:\n{}\n\n",
        //     serde_json::to_string_pretty(&c2.clone()).unwrap()
        // );
        // println!("[c2]:\n{}\n\n", c2.get_build_string().unwrap());

        let mut new_plan = c1.clone();
        new_plan.resolve_phase_names();
        let mut c2 = c2.clone();
        c2.resolve_phase_names();

        new_plan.build_image = c2.build_image;
        let static_assets = new_plan
            .static_assets
            .get_or_insert(StaticAssets::default());
        static_assets.extend(c2.static_assets.unwrap_or_default());

        let variables = new_plan
            .variables
            .get_or_insert(EnvironmentVariables::default());
        variables.extend(c2.variables.unwrap_or_default());

        for (name, c2_phase) in c2.phases.clone().unwrap_or_default() {
            let phase = new_plan.remove_phase(&name);
            let phase = phase.unwrap_or_else(|| {
                let mut phase = Phase::new(name.clone());
                if name == "install" {
                    phase.depends_on_phase("setup");
                } else if name == "build" {
                    phase.depends_on_phase("install");
                };

                phase
            });

            let merged_phase = Phase::merge(&phase, &c2_phase);
            new_plan.add_phase(merged_phase);
        }

        let new_start_phase = StartPhase::merge(
            &new_plan.start_phase.clone().unwrap_or_default(),
            &c2.start_phase.clone().unwrap_or_default(),
        );
        new_plan.set_start_phase(new_start_phase);

        // println!("[c3]:\n{}\n\n", new_plan.get_build_string().unwrap());
        // println!("\n\n=== DONE ===\n");

        new_plan.resolve_phase_names();
        new_plan
    }
}

impl Mergeable for Phase {
    fn merge(c1: &Phase, c2: &Phase) -> Phase {
        let mut phase = c1.clone();
        let c2 = c2.clone();
        phase.nixpacks_archive = c2
            .nixpacks_archive
            .or_else(|| phase.nixpacks_archive.clone());

        phase.cmds = extract_auto_from_vec(phase.cmds.clone(), c2.cmds);
        phase.depends_on = extract_auto_from_vec(phase.depends_on.clone(), c2.depends_on);
        phase.nix_pkgs = extract_auto_from_vec(phase.nix_pkgs.clone(), c2.nix_pkgs);
        phase.nix_libs = extract_auto_from_vec(phase.nix_libs.clone(), c2.nix_libs);
        phase.apt_pkgs = extract_auto_from_vec(phase.apt_pkgs.clone(), c2.apt_pkgs);
        phase.nix_overlays = extract_auto_from_vec(phase.nix_overlays.clone(), c2.nix_overlays);
        phase.only_include_files =
            extract_auto_from_vec(phase.only_include_files.clone(), c2.only_include_files);
        phase.cache_directories =
            extract_auto_from_vec(phase.cache_directories.clone(), c2.cache_directories);
        phase.paths = extract_auto_from_vec(phase.paths.clone(), c2.paths);

        phase
    }
}

impl Mergeable for StartPhase {
    fn merge(c1: &StartPhase, c2: &StartPhase) -> StartPhase {
        let mut start_phase = c1.clone();
        let c2 = c2.clone();
        start_phase.cmd = c2.cmd.or_else(|| start_phase.cmd.clone());
        start_phase.run_image = c2.run_image.or_else(|| start_phase.run_image.clone());
        start_phase.only_include_files = extract_auto_from_vec(
            start_phase.only_include_files.clone(),
            c2.only_include_files,
        );
        start_phase
    }
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

/// Replaces the "..." or "@auto" in the `replacer` with the values from the `original`
///
/// ```
/// let arr = extract_auto_from_vec(Some(vec!["a", "b", "c"]), Some(vec!["x", "...", "z"]))
/// assert_eq!(Some(vec!["x", "a", "b", "c", "z"]), arr);
/// ```
fn extract_auto_from_vec(
    original: Option<Vec<String>>,
    replacer: Option<Vec<String>>,
) -> Option<Vec<String>> {
    if let Some(replacer) = replacer {
        let modified = replacer
            .into_iter()
            .map(|x| vec![x])
            .flat_map(|pkgs| {
                let v = pkgs[0].clone();
                if v == "@auto" || v == "..." {
                    original.clone().unwrap_or_default()
                } else {
                    pkgs
                }
            })
            .collect::<Vec<_>>();

        Some(modified)
    } else {
        original
    }
}

impl Default for BuildPlan {
    fn default() -> Self {
        Self {
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

    #[test]
    fn test_merge_plans() {
        let plan1 = BuildPlan {
            phases: Some(Phases::from([
                (
                    "setup".to_string(),
                    Phase {
                        nix_pkgs: Some(vec!["nodejs".to_string()]),
                        apt_pkgs: Some(vec!["wget".to_string()]),
                        ..Default::default()
                    },
                ),
                (
                    "build".to_string(),
                    Phase {
                        cmds: Some(vec!["yarn run build".to_string()]),
                        ..Default::default()
                    },
                ),
            ])),
            start_phase: Some(StartPhase {
                cmd: Some("yarn start1".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let plan2 = BuildPlan {
            phases: Some(Phases::from([
                (
                    "setup".to_string(),
                    Phase {
                        nix_pkgs: Some(vec!["...".to_string(), "cowsay".to_string()]),
                        apt_pkgs: Some(vec!["sl".to_string()]),
                        ..Default::default()
                    },
                ),
                (
                    "install".to_string(),
                    Phase {
                        cmds: Some(vec!["yarn install".to_string()]),
                        ..Default::default()
                    },
                ),
                (
                    "build".to_string(),
                    Phase {
                        cmds: Some(vec![
                            "...".to_string(),
                            "yarn run optimize-assets".to_string(),
                        ]),
                        ..Default::default()
                    },
                ),
            ])),
            start_phase: Some(StartPhase {
                cmd: Some("yarn start2".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut expected = BuildPlan {
            phases: Some(Phases::from([
                (
                    "setup".to_string(),
                    Phase {
                        nix_pkgs: Some(vec!["nodejs".to_string(), "cowsay".to_string()]),
                        apt_pkgs: Some(vec!["sl".to_string()]),
                        ..Default::default()
                    },
                ),
                (
                    "install".to_string(),
                    Phase {
                        depends_on: Some(vec!["setup".to_string()]),
                        cmds: Some(vec!["yarn install".to_string()]),
                        ..Default::default()
                    },
                ),
                (
                    "build".to_string(),
                    Phase {
                        cmds: Some(vec![
                            "yarn run build".to_string(),
                            "yarn run optimize-assets".to_string(),
                        ]),
                        ..Default::default()
                    },
                ),
            ])),
            start_phase: Some(StartPhase {
                cmd: Some("yarn start2".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        expected.resolve_phase_names();

        let merged = BuildPlan::merge(&plan1, &plan2);

        assert_eq!(expected, merged);
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
