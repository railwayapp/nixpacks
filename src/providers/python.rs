use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    chain,
    nixpacks::{
        app::App,
        environment::Environment,
        nix::NixConfig,
        phase::{InstallPhase, SetupPhase, StartPhase},
    },
    Pkg,
};

use super::Provider;

pub struct PythonProvider {}
impl Provider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn detect(&self, app: &crate::nixpacks::app::App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.py")
            || app.includes_file("requirements.txt")
            || app.includes_file("pyproject.toml"))
    }

    fn setup(
        &self,
        _app: &App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> Result<SetupPhase> {
        Ok(SetupPhase::new(NixConfig::new(vec![Pkg::new(
            "pkgs.python38",
        )])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<InstallPhase> {
        if app.includes_file("requirements.txt") {
            return Ok(InstallPhase::new(
                "python -m ensurepip && python -m pip install -r requirements.txt".to_string(),
            ));
        } else if app.includes_file("pyproject.toml") {
            return Ok(InstallPhase::new("python -m ensurepip && python -m pip install --upgrade build setuptools && python -m pip install .".to_string()));
        }
        Ok(InstallPhase::default())
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<StartPhase> {
        if app.includes_file("pyproject.toml") {
            if let Ok(meta) = self.parse_pyproject(app) {
                if let Some(entry_point) = meta.entry_point {
                    return Ok(StartPhase::new(match entry_point {
                        EntryPoint::Command(cmd) => cmd,
                        EntryPoint::Module(module) => format!("python -m {}", module),
                    }));
                }
            }
        }
        // falls through
        if app.includes_file("main.py") {
            return Ok(StartPhase::new("python main.py".to_string()));
        }

        Ok(StartPhase::default())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct PyProject {
    pub project: Option<ProjectDecl>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ProjectDecl {
    pub name: Option<String>,
    pub packages: Option<Vec<String>>,
    pub py_modules: Option<Vec<String>>,
    pub entry_points: Option<HashMap<String, String>>,
}

#[allow(dead_code)]
struct ProjectMeta {
    pub project_name: Option<String>,
    pub module_name: Option<String>,
    pub entry_point: Option<EntryPoint>,
}

#[allow(dead_code)]
enum EntryPoint {
    Command(String),
    Module(String),
}

impl PythonProvider {
    fn read_pyproject(&self, app: &App) -> Result<Option<PyProject>> {
        if app.includes_file("pyproject.toml") {
            return Ok(Some(
                app.read_toml("pyproject.toml")
                    .context("Reading pyproject.toml")?,
            ));
        }
        Ok(None)
    }
    fn parse_project(&self, project: &PyProject) -> ProjectMeta {
        let project_name = project
            .project
            .as_ref()
            .and_then(|proj| proj.name.as_ref())
            .map(|name| name.to_owned());

        let module_name = chain!(project.project.clone() =>
            (
                |proj| proj.packages,
                |pkgs| pkgs.get(0).cloned()
            );
            (
                |proj| proj.py_modules,
                |mods| mods.get(0).cloned()
            );
            (
                |_| project_name.to_owned()
            )
        );

        let entry_point = module_name.to_owned().map(EntryPoint::Module);

        ProjectMeta {
            project_name,
            module_name,
            entry_point,
        }
    }
    fn parse_pyproject(&self, app: &App) -> Result<ProjectMeta> {
        Ok(self.parse_project(
            &(self
                .read_pyproject(app)?
                .ok_or_else(|| anyhow::anyhow!("failed to load pyproject.toml"))?),
        ))
    }
}
