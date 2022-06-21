use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

pub mod generator;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: Option<String>,
    pub setup: Option<SetupPhase>,
    pub install: Option<InstallPhase>,
    pub build: Option<BuildPhase>,
    pub start: Option<StartPhase>,
    pub variables: Option<EnvironmentVariables>,
    pub static_assets: Option<StaticAssets>,
}

pub trait PlanGenerator {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan>;
}

impl BuildPlan {
    pub fn get_build_string(&self) -> String {
        let setup_phase = self.setup.clone();
        let nix_pkgs = setup_phase
            .clone()
            .unwrap_or_default()
            .pkgs
            .iter()
            .map(|pkg| pkg.to_pretty_string())
            .collect::<Vec<_>>();
        let apt_pkgs = setup_phase
            .clone()
            .unwrap_or_default()
            .apt_pkgs
            .clone()
            .unwrap_or_default();
        let pkgs = [nix_pkgs, apt_pkgs].concat();
        let packages_string = get_phase_string("Packages", Some(pkgs.join("\n    -> ")));
        let install_phase = self.install.clone();
        let install_string = get_phase_string(
            "Install",
            install_phase.map(|install| install.cmds.unwrap_or_default().join("\n    -> ")),
        );

        let build_phase = self.build.clone();
        let build_string = get_phase_string(
            "Build",
            build_phase.map(|build| build.cmds.unwrap_or_default().join("\n    -> ")),
        );

        let start_phase = self.start.clone();
        let start_string = get_phase_string("Start", start_phase.and_then(|start| start.cmd));

        return formatdoc! {"
          {packages_string}
          {install_string}
          {build_string}
          {start_string}",
            packages_string=packages_string,
            install_string=install_string,
            build_string=build_string,
        start_string=start_string};
    }
}

fn get_phase_string(phase: &str, content: Option<String>) -> String {
    match &content {
        Some(content) => {
            if content.is_empty() {
                return format!("=> {}\n    -> Skipping", phase);
            }
            format!("=> {}\n    -> {}", phase, content.trim())
        }
        None => {
            format!("=> {}\n    -> Skipping", phase)
        }
    }
}
