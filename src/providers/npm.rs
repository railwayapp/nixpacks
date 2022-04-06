use std::collections::HashMap;

use super::{Pkg, Provider};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub struct NpmProvider {}

impl Provider for NpmProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn pkgs(&self, app: &App, _env: &Environment) -> Result<Vec<Pkg>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = get_nix_node_pkg(&package_json)?
            .and_then(|v| Some(Pkg::new(&v)))
            .unwrap_or_else(|| Pkg::new(&"nodejs".to_string()));

        Ok(vec![Pkg::new("pkgs.stdenv"), node_pkg])
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("npm install".to_string()))
    }

    fn suggested_build_cmd(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("build").is_some() {
                return Ok(Some("npm run build".to_string()));
            }
        }

        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get("start").is_some() {
                return Ok(Some("npm run start".to_string()));
            }
        }

        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                return Ok(Some(format!("node {}", main)));
            }
        }
        if app.includes_file("index.js") {
            return Ok(Some(String::from("node index.js")));
        }

        Ok(None)
    }

    fn get_environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        let mut variables = EnvironmentVariables::default();
        variables.insert("NODE_ENV".to_string(), "production".to_string());
        variables.insert("NPM_CONFIG_PRODUCTION".to_string(), "false".to_string());

        Ok(variables)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub scripts: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub main: Option<String>,
}

const AVAILABLE_NODE_VERSIONS: &'static [u32] = &[10, 12, 14, 16, 17];

fn version_number_to_pkg(version: &u32) -> Result<Option<String>> {
    if AVAILABLE_NODE_VERSIONS.contains(version) {
        Ok(Some(format!("nodejs-{}_x", version)))
    } else {
        bail!("Node version {} is not available", version);
    }
}

pub fn get_nix_node_pkg(package_json: &PackageJson) -> Result<Option<String>> {
    let node_version = package_json
        .engines
        .as_ref()
        .and_then(|engines| engines.get("node"));

    let node_version = match node_version {
        Some(node_version) => node_version,
        None => return Ok(None),
    };

    if node_version == "*" {
        // Any version will work, use latest
        return Ok(None);
    }

    match node_version.parse::<u32>() {
        Ok(version) => return version_number_to_pkg(&version),
        Err(_e) => {}
    }

    println!("VERSION: {}", node_version);

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    fn engines_node(version: &str) -> Option<HashMap<String, String>> {
        Some(HashMap::from([("node".to_string(), version.to_string())]))
    }

    #[test]
    fn test_no_engines() -> Result<()> {
        assert!(get_nix_node_pkg(&PackageJson {
            name: String::default(),
            main: None,
            scripts: None,
            engines: None
        })?
        .is_none());

        Ok(())
    }

    #[test]
    fn test_star_engine() -> Result<()> {
        assert!(get_nix_node_pkg(&PackageJson {
            name: String::default(),
            main: None,
            scripts: None,
            engines: engines_node("*")
        })?
        .is_none());

        Ok(())
    }

    #[test]
    fn test_simple_engine() -> Result<()> {
        assert_eq!(
            get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                engines: engines_node("14"),
            })?,
            Some("nodejs-14_x".to_string())
        );

        Ok(())
    }
}
