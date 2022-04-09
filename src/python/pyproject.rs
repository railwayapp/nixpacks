use anyhow::{Context, Result};

use crate::{chain, nixpacks::app::App};

pub struct ProjectMeta {
    pub project_name: Option<String>,
    pub module_name: Option<String>,
    pub entry_point: Option<EntryPoint>,
}

pub enum EntryPoint {
    Command(String),
    Module(String),
}

pub fn parse(app: &App) -> Result<ProjectMeta> {
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
