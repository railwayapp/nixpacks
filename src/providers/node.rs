use std::collections::HashMap;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

const AVAILABLE_NODE_VERSIONS: &[u32] = &[10, 12, 14, 16, 17];
pub const DEFAULT_NODE_PKG_NAME: &'static &str = &"nodejs";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PackageJson {
    pub name: String,
    pub scripts: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub workspaces: Option<Vec<String>>,
    pub main: Option<String>,
}

pub struct NodeProvider {}

impl Provider for NodeProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn setup(&self, app: &App, env: &Environment) -> Result<Option<SetupPhase>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, env)?;

        if NodeProvider::get_package_manager(app)? == "pnpm" {
            let mut pnpm_pkg = Pkg::new("nodePackages.pnpm");
            // Only override the node package if not the default one
            if node_pkg.name != *DEFAULT_NODE_PKG_NAME {
                pnpm_pkg = pnpm_pkg.set_override("nodejs", node_pkg.name.as_str());
            }
            return Ok(Some(SetupPhase::new(vec![node_pkg, pnpm_pkg])));
        } else if NodeProvider::get_package_manager(app)? == "yarn" {
            let mut yarn_pkg = Pkg::new("yarn");
            // Only override the node package if not the default one
            if node_pkg.name != *DEFAULT_NODE_PKG_NAME {
                yarn_pkg = yarn_pkg.set_override("nodejs", node_pkg.name.as_str());
            }
            return Ok(Some(SetupPhase::new(vec![node_pkg, yarn_pkg])));
        }

        Ok(Some(SetupPhase::new(vec![node_pkg])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_cmd = "npm i";
        if NodeProvider::get_package_manager(app)? == "pnpm" {
            install_cmd = "pnpm i --frozen-lockfile"
        } else if NodeProvider::get_package_manager(app)? == "yarn" {
            if app.includes_file(".yarnrc.yml") {
                install_cmd = "yarn set version berry && yarn install --immutable --check-cache"
            } else {
                install_cmd = "yarn install --frozen-lockfile --production=false"
            }
        } else if app.includes_file("package-lock.json") {
            install_cmd = "npm ci"
        }
        Ok(Some(InstallPhase::new(install_cmd.to_string())))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        if NodeProvider::has_script(app, "build")? {
            let pkg_manager = NodeProvider::get_package_manager(app)?;
            Ok(Some(BuildPhase::new(pkg_manager + " run build")))
        } else {
            Ok(None)
        }
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = NodeProvider::get_start_cmd(app)? {
            let pkg_manager = NodeProvider::get_package_manager(app)?;
            Ok(Some(StartPhase::new(
                start_cmd.replace("npm", &pkg_manager),
            )))
        } else {
            Ok(None)
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(NodeProvider::get_node_environment_variables()))
    }
}

impl NodeProvider {
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
        if NodeProvider::has_script(app, "start")? {
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
    pub fn get_nix_node_pkg(package_json: &PackageJson, environment: &Environment) -> Result<Pkg> {
        let env_node_version = environment.get_config_variable("NODE_VERSION");

        let pkg_node_version = package_json
            .engines
            .as_ref()
            .and_then(|engines| engines.get("node"));

        let node_version = pkg_node_version.or(env_node_version);

        let node_version = match node_version {
            Some(node_version) => node_version,
            None => return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME)),
        };

        // Any version will work, use latest
        if node_version == "*" {
            return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME));
        }

        // Parse `12` or `12.x` into nodejs-12_x
        let re = Regex::new(r"^(\d+)\.?[x|X]?$").unwrap();
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

    pub fn get_package_manager(app: &App) -> Result<String> {
        let mut pkg_manager = "npm";
        if app.includes_file("pnpm-lock.yaml") {
            pkg_manager = "pnpm";
        } else if app.includes_file("yarn.lock") {
            pkg_manager = "yarn";
        }
        Ok(pkg_manager.to_string())
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
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: None
                },
                &Environment::default()
            )?,
            Pkg::new(DEFAULT_NODE_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_star_engine() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: engines_node("*")
                },
                &Environment::default()
            )?,
            Pkg::new(DEFAULT_NODE_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_simple_engine() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: engines_node("14"),
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_simple_engine_x() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: engines_node("12.x"),
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-12_x")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: engines_node("14.X"),
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_engine_range() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: engines_node(">=14.10.3 <16"),
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_environment_variable() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: String::default(),
                    main: None,
                    scripts: None,
                    workspaces: None,
                    engines: None,
                },
                &Environment::new(HashMap::from([(
                    "NIXPACKS_NODE_VERSION".to_string(),
                    "14".to_string()
                )]))
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_engine_invalid_version() -> Result<()> {
        assert!(NodeProvider::get_nix_node_pkg(
            &PackageJson {
                name: String::default(),
                main: None,
                scripts: None,
                workspaces: None,
                engines: engines_node("15"),
            },
            &Environment::default()
        )
        .is_err());

        Ok(())
    }
}
