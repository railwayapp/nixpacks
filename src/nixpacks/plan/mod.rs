use self::{
    merge::Mergeable,
    phase::{Phase, Phases, StartPhase},
    topological_sort::topological_sort,
};
use super::images::DEFAULT_BASE_IMAGE;
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
#[derive(PartialEq, Eq, Default, Debug, Serialize, Deserialize, Clone)]
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
    pub fn new(phases: &[Phase], start_phase: Option<StartPhase>) -> Self {
        Self {
            phases: Some(phases.iter().map(|p| (p.get_name(), p.clone())).collect()),
            start_phase,
            ..Default::default()
        }
    }

    pub fn from_toml<S: Into<String>>(toml: S) -> Result<Self> {
        let mut plan: BuildPlan = toml::from_str(&toml.into())?;
        plan.resolve_phase_names();
        Ok(plan)
    }

    pub fn from_json<S: Into<String>>(json: S) -> Result<Self> {
        let mut plan: BuildPlan = serde_json::from_str(&json.into())?;
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
        for (_, mut phase) in phases {
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
        let phases = self.phases.get_or_insert(BTreeMap::default());
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

        BuildPlan::new(&phases, start)
    }

    pub fn pin(&mut self) {
        self.providers = Some(Vec::new());
        if self.build_image.is_none() {
            self.build_image = Some(DEFAULT_BASE_IMAGE.to_string());
        }

        self.resolve_phase_names();
        let phases = self.phases.get_or_insert(Phases::default());
        for (_, phase) in phases.iter_mut() {
            phase.pin();
        }

        if let Some(start) = &mut self.start_phase {
            start.pin();
        }
    }

    pub fn prefix_phases(&mut self, prefix: &str) {
        if let Some(phases) = self.phases.clone() {
            self.resolve_phase_names();
            let mut new_phases = Phases::default();

            for (_, phase) in phases {
                let mut new_phase = phase.clone();
                new_phase.prefix_name(prefix);
                new_phase.prefix_dependencies(prefix);
                new_phases.insert(new_phase.get_name(), new_phase);
            }

            self.phases = Some(new_phases);
        }
    }

    pub fn merge_plans(plans: &[BuildPlan]) -> BuildPlan {
        plans.iter().fold(BuildPlan::default(), |acc, plan| {
            BuildPlan::merge(&acc, plan)
        })
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

        let plan = BuildPlan::new(&vec![setup, install, build, another], None);

        // let phases = topological_sort(plan.get_phases_with_dependencies("build")).unwrap();
        let build_phase = plan.get_phases_with_dependencies("build");
        let phases = build_phase.values();

        assert_eq!(phases.len(), 3);
    }

    #[test]
    fn test_pin_build_plan() {
        let mut plan = BuildPlan::from_toml(
            r#"
            [phases.setup]
            nixPkgs = ["nodejs", "@auto", "yarn"]

            [phases.build]
            cmds = ["yarn run build", "...", "yarn run optimize-assets"]

            [start]
            cmd = "yarn run start"
            "#,
        )
        .unwrap();

        plan.pin();
        assert_eq!(
            plan.get_phase("setup").unwrap().nix_pkgs,
            Some(vec!["nodejs".to_string(), "yarn".to_string()])
        );
        assert!(plan.get_phase("setup").unwrap().nixpkgs_archive.is_some());
    }
}
