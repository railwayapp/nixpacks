use anyhow::{Result, Context};

use crate::nixpacks::app::App;

pub struct ProjectMeta {
    pub project_name: Option<String>,
    pub module_name: Option<String>,
    pub entry_point: Option<EntryPoint>
}

pub enum EntryPoint {
    Command(String),
    Module(String)
}

pub fn parse(app: &App) -> Result<ProjectMeta> {
    if !app.includes_file("pyproject.toml") {
        return Err(anyhow::anyhow!("no project.toml found"));
    }
    let pyproject: toml::Value = app.read_toml("pyproject.toml").context("Reading pyproject.toml")?;

    let project_name = pyproject.get("project")
        .and_then(|project| project.get("name"))
        .and_then(|x| x.as_str())
        .and_then(|s| Some(s.to_string()));

    let module_name = pyproject.get("project")
        .and_then(|project| project.get("packages"))
        .and_then(|packages| packages.as_array())
        .and_then(|packages| packages.get(0))
        .and_then(|x| x.as_str())
        .and_then(|s| Some(s.to_string()))
        .or_else(|| pyproject
            .get("project")
            .and_then(|project| project.get("py-modules"))
            .and_then(|modules| modules.as_array())
            .and_then(|modules| modules.get(0))
            .and_then(|module| module.as_str())
            .and_then(|str| Some(str.to_string()))
        )
        .or_else(|| project_name.to_owned());
    
    let entry_point = pyproject.get("project")
        .and_then(|project| project.get("scripts"))
        .and_then(|scripts| scripts.as_table())
        .and_then(|scripts| Some(scripts.keys()))
        .and_then(|mut cmds| cmds.nth(0))
        .and_then(|cmd| Some(EntryPoint::Command(cmd.to_string())))
        .or_else(|| module_name.to_owned().and_then(|module| Some(EntryPoint::Module(module))));
    
    Ok(ProjectMeta {
        project_name,
        module_name,
        entry_point
    })
}