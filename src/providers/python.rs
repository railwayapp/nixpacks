use anyhow::{Result, Context};

use crate::{
    nixpacks::{app::App, environment::Environment, nix::NixConfig},
    Pkg, chain,
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

    fn install_cmd(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        if app.includes_file("requirements.txt") {
            return Ok(Some(
                "python -m ensurepip && python -m pip install -r requirements.txt".to_string(),
            ));
        } else if app.includes_file("pyproject.toml") {
            return Ok(Some("python -m ensurepip && python -m pip install --upgrade build setuptools && python -m pip install .".to_string()));
        }
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        if app.includes_file("pyproject.toml") {
            if let Ok(meta) = parse_project(app) {
                if let Some(entry_point) = meta.entry_point {
                    return match entry_point {
                        EntryPoint::Command(cmd) => Ok(Some(cmd)),
                        EntryPoint::Module(module) => {
                            Ok(Some(format!("python -m {}", module)))
                        }
                    };
                }
            }
        }
        // falls through
        if app.includes_file("main.py") {
            return Ok(Some("python main.py".to_string()));
        }
        Ok(None)
    }

    fn nix_config(
        &self,
        _app: &App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> Result<crate::nixpacks::nix::NixConfig> {
        Ok(NixConfig::new(vec![Pkg::new("pkgs.python38")]))
    }
}

#[allow(dead_code)]
struct ProjectMeta {
    pub project_name: Option<String>,
    pub module_name: Option<String>,
    pub entry_point: Option<EntryPoint>,
}

enum EntryPoint {
    Command(String),
    Module(String),
}

fn parse_project(app: &App) -> Result<ProjectMeta> {
    if !app.includes_file("pyproject.toml") {
        return Err(anyhow::anyhow!("no project.toml found"));
    }
    let pyproject: toml::Value = app
        .read_toml("pyproject.toml")
        .context("Reading pyproject.toml")?;
    let project = pyproject.get("project");
    let project_name = chain!(project =>
        |proj| proj.get("name"),
        |name| name.as_str(),
        |name| Some(name.to_string())
    );

    let module_name = chain!(project =>
        (
            |proj| proj.get("packages"),
            |pkgs| pkgs.as_array(),
            |pkgs| pkgs.get(0),
            |package| package.as_str(),
            |name| Some(name.to_string())
        );
        (
            |proj| proj.get("py-modules"),
            |mods| mods.as_array(),
            |mods| mods.get(0),
            |module| module.as_str(),
            |name| Some(name.to_string())
        );
        (
            |_| project_name.to_owned()
        )
    );

    let entry_point = chain!(project =>
        (
            |project| project.get("scripts"),
            |scripts| scripts.as_table(),
            |scripts| Some(scripts.keys()),
            |mut cmds| cmds.next(),
            |cmd| Some(EntryPoint::Command(cmd.to_string()))
        );
        (
            |_| module_name.to_owned(),
            |module| Some(EntryPoint::Module(module))
        )
    );

    Ok(ProjectMeta {
        project_name,
        module_name,
        entry_point,
    })
}
