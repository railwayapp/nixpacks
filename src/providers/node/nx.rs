// Code relating to NX Monorepos

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::nixpacks::{app::App, environment::Environment};
use crate::providers::node::NodeProvider;

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct NxJson {
    #[serde(alias = "defaultProject")]
    pub default_project: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct ProjectJson {
    pub targets: Targets,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct Targets {
    pub build: Target,
    pub start: Option<Target>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct Target {
    pub executor: String,
    pub options: Option<NxTargetOptions>,
    pub configurations: Option<Configuration>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct NxTargetOptions {
    #[serde(alias = "outputPath")]
    pub output_path: Option<Value>,
    pub main: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct Configuration {
    pub production: Option<Value>,
}

pub struct Nx {}

const NX_APP_NAME_ENV_VAR: &str = "NX_APP_NAME";

impl Nx {
    pub fn is_nx_monorepo(app: &App, env: &Environment) -> bool {
        // Only consider an Nx app if an nx app name and project path can be found
        if let Some(nx_app_name) = Nx::get_nx_app_name(app, env) {
            return app.includes_file("nx.json")
                && Nx::get_nx_project_json_for_app(app, &nx_app_name).is_ok();
        }

        false
    }

    pub fn get_nx_app_name(app: &App, env: &Environment) -> Option<String> {
        // First, check if explicitly set via environment variable
        if let Some(app_name) = env.get_config_variable(NX_APP_NAME_ENV_VAR) {
            return Some(app_name);
        }

        // Second, check nx.json for default project
        if let Ok(nx_json) = app.read_json::<NxJson>("nx.json") {
            if let Some(default_project) = nx_json.default_project {
                return Some(default_project);
            }
        }

        // Third, try to auto-detect by looking for apps with valid configurations
        if app.includes_directory("apps") {
            // Look for directories in apps/ that have either project.json or package.json with nx config
            if let Ok(app_dirs) = app.find_directories("apps/*") {
                for app_dir in app_dirs {
                    if let Some(app_name) = app_dir.file_name().and_then(|n| n.to_str()) {
                        let app_path = format!("apps/{app_name}");
                        // Check if this app has a valid project.json
                        let project_json_path = format!("{app_path}/project.json");
                        if app.includes_file(&project_json_path) {
                            return Some(app_name.to_string());
                        }
                        // Check if this app has a package.json with nx targets
                        let package_json_path = format!("{app_path}/package.json");
                        if app.includes_file(&package_json_path) {
                            if let Ok(pkg_json) =
                                app.read_json::<serde_json::Value>(&package_json_path)
                            {
                                if pkg_json
                                    .get("nx")
                                    .and_then(|nx| nx.get("targets"))
                                    .is_some()
                                {
                                    return Some(app_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_nx_project_json_for_app(app: &App, nx_app_name: &String) -> Result<ProjectJson> {
        // Try project.json (old style NX configuration)
        let project_path = format!("./apps/{nx_app_name}/project.json");
        if let Ok(project_json) = app.read_json::<ProjectJson>(&project_path) {
            return Ok(project_json);
        }

        // Try package.json (new NX 20+ style configuration)
        let package_path = format!("./apps/{nx_app_name}/package.json");
        if let Ok(pkg_json) = app.read_json::<serde_json::Value>(&package_path) {
            if let Some(nx) = pkg_json.get("nx") {
                // If targets exist, use them
                if let Some(targets) = nx.get("targets") {
                    let targets: Targets = serde_json::from_value(targets.clone())?;
                    return Ok(ProjectJson { targets });
                }
            }
        }

        Err(anyhow::anyhow!(
            "No build/start targets found for NX app '{}'. For NX 20+, ensure your app's package.json contains an 'nx.targets' section with 'build' and 'start' targets. For older NX versions, ensure your app has a project.json file with target definitions. You can also set the {} environment variable to specify the app name explicitly.",
            nx_app_name,
            NX_APP_NAME_ENV_VAR
        ))
    }

    pub fn get_nx_output_path(app: &App, nx_app_name: &String) -> Result<String> {
        let project_json = Nx::get_nx_project_json_for_app(app, nx_app_name)?;
        if let Some(options) = project_json.targets.build.options {
            if let Some(output_path) = options.output_path {
                if let Some(the_output_path) = output_path.as_str() {
                    return Ok(the_output_path.to_string());
                }
            }
        }

        Ok(format!("dist/apps/{nx_app_name}"))
    }

    pub fn get_nx_build_cmd(app: &App, env: &Environment) -> Option<String> {
        Nx::get_nx_app_name(app, env).map(|nx_app_name| {
            format!(
                "{} nx run {nx_app_name}:build:production",
                NodeProvider::get_package_manager_dlx_command(app)
            )
        })
    }

    pub fn get_nx_start_cmd(app: &App, env: &Environment) -> Result<Option<String>> {
        if !Nx::is_nx_monorepo(app, env) {
            return Ok(None);
        }

        if let Some(nx_app_name) = Nx::get_nx_app_name(app, env) {
            let output_path = Nx::get_nx_output_path(app, &nx_app_name)?;
            let project_json = Nx::get_nx_project_json_for_app(app, &nx_app_name)?;

            if let Some(start_target) = project_json.targets.start {
                if let Some(configurations) = start_target.configurations {
                    if configurations.production.is_some() {
                        return Ok(Some(format!(
                            "{} nx run {nx_app_name}:start:production",
                            NodeProvider::get_package_manager_dlx_command(app)
                        )));
                    }
                }
                return Ok(Some(format!(
                    "{} nx run {nx_app_name}:start",
                    NodeProvider::get_package_manager_dlx_command(app)
                )));
            }

            if project_json.targets.build.executor == "@nx/next:build"
                || project_json.targets.build.executor == "@nrwl/next:build"
            {
                return Ok(Some(format!("cd {output_path} && npm run start")));
            }

            if let Some(options) = project_json.targets.build.options {
                if let Some(main_path) = options.main {
                    let current_path = PathBuf::from(main_path);
                    let file_name = current_path.file_stem().unwrap().to_str().unwrap();

                    return Ok(Some(format!("node {output_path}/{file_name}.js")));
                }
            }
            return Ok(Some(format!("node {output_path}/index.js")));
        }

        Ok(None)
    }
}
