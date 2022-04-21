use std::collections::HashMap;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

const AVAILABLE_NODE_VERSIONS: &[u32] = &[10, 12, 14, 16, 17];
const DEFAULT_NODE_PKG_NAME: &'static &str = &"pkgs.nodejs";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PackageJson {
    pub name: String,
    pub scripts: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub workspaces: Option<Vec<String>>,
    pub main: Option<String>,
}

pub struct NpmProvider {}

impl Provider for NpmProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = NpmProvider::get_nix_node_pkg(&package_json)?;

        Ok(Some(SetupPhase::new(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            node_pkg,
        ]))))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("npm install".to_string());

        // Installing node modules only depends on package.json and lock file
        install_phase.add_file_dependency("package.json".to_string());
        if app.includes_file("package-lock.json") {
            install_phase.add_file_dependency("package-lock.json".to_string());
        }

        Ok(Some(install_phase))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        if NpmProvider::has_script(app, "build")? {
            Ok(Some(BuildPhase::new("npm run build".to_string())))
        } else {
            Ok(None)
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = NpmProvider::get_start_cmd(app)? {
            Ok(Some(StartPhase::new(start_cmd)))
        } else {
            Ok(None)
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(NpmProvider::get_node_environment_variables()))
    }
}

impl NpmProvider {
    pub fn get_node_environment_variables() -> EnvironmentVariables {
        EnvironmentVariables::from([
            ("NODE_ENV".to_string(), "production".to_string()),
            ("NPM_CONFIG_PRODUCTION".to_string(), "false".to_string()),
        ])
    }

    pub fn has_script(app: &App, script: &str) -> Result<bool> {
        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(scripts) = package_json.scripts {
            if scripts.get(script).is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get_start_cmd(app: &App) -> Result<Option<String>> {
        if NpmProvider::has_script(app, "start")? {
            return Ok(Some("npm run start".to_string()));
        }

        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                return Ok(Some(format!("node {}", main)));
            }
        }

        if app.includes_file("index.js") {
            return Ok(Some("node index.js".to_string()));
        }

        Ok(None)
    }

    /// Parses the package.json engines field and returns a Nix package if available
    pub fn get_nix_node_pkg(package_json: &PackageJson) -> Result<Pkg> {
        let node_version = package_json
            .engines
            .as_ref()
            .and_then(|engines| engines.get("node"));

        let node_version = match node_version {
            Some(node_version) => node_version,
            None => return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME)),
        };

        // Any version will work, use latest
        if node_version == "*" {
            return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME));
        }

        // Parse `12` or `12.x` into nodejs-12_x
        let re = Regex::new(r"^(\d+)\.?x?$").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, node_version)? {
            return Ok(Pkg::new(node_pkg.as_str()));
        }

        // Parse `>=14.10.3 <16` into nodejs-14_x
        let re = Regex::new(r"^>=(\d+)").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, node_version)? {
            return Ok(Pkg::new(node_pkg.as_str()));
        }

        Ok(Pkg::new(DEFAULT_NODE_PKG_NAME))
    }
}

fn version_number_to_pkg(version: &u32) -> Result<Option<String>> {
    if AVAILABLE_NODE_VERSIONS.contains(version) {
        Ok(Some(format!("nodejs-{}_x", version)))
    } else {
        bail!("Node version {} is not available", version);
    }
}

fn parse_regex_into_pkg(re: &Regex, node_version: &str) -> Result<Option<String>> {
    let matches: Vec<_> = re.captures_iter(node_version).collect();
    if let Some(m) = matches.get(0) {
        match m[1].parse::<u32>() {
            Ok(version) => return version_number_to_pkg(&version),
            Err(_e) => {}
        }
    }

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
        assert_eq!(
            NpmProvider::get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: None
            })?,
            Pkg::new(DEFAULT_NODE_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_star_engine() -> Result<()> {
        assert_eq!(
            NpmProvider::get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: engines_node("*")
            })?,
            Pkg::new(DEFAULT_NODE_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_simple_engine() -> Result<()> {
        assert_eq!(
            NpmProvider::get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: engines_node("14"),
            })?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_simple_engine_x() -> Result<()> {
        assert_eq!(
            NpmProvider::get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: engines_node("12.x"),
            })?,
            Pkg::new("nodejs-12_x")
        );

        Ok(())
    }

    #[test]
    fn test_engine_range() -> Result<()> {
        assert_eq!(
            NpmProvider::get_nix_node_pkg(&PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: engines_node(">=14.10.3 <16"),
            })?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_engine_invalid_version() -> Result<()> {
        assert!(NpmProvider::get_nix_node_pkg(&PackageJson {
            name: String::default(),
            main: None,
            scripts: None,
            workspaces: None,
            engines: engines_node("15"),
        })
        .is_err());

        Ok(())
    }
}
