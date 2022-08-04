use std::collections::{HashMap, HashSet};

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const NODE_OVERLAY: &str = "https://github.com/railwayapp/nix-npm-overlay/archive/main.tar.gz";

const DEFAULT_NODE_PKG_NAME: &'static &str = &"nodejs";
const AVAILABLE_NODE_VERSIONS: &[u32] = &[14, 16, 18];

const YARN_CACHE_DIR: &'static &str = &"/usr/local/share/.cache/yarn/v6";
const PNPM_CACHE_DIR: &'static &str = &"/root/.cache/pnpm";
const NPM_CACHE_DIR: &'static &str = &"/root/.npm";
const BUN_CACHE_DIR: &'static &str = &"/root/.bun";
const CYPRESS_CACHE_DIR: &'static &str = &"/root/.cache/Cypress";
const NODE_MODULES_CACHE_DIR: &'static &str = &"node_modules/.cache";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PackageJson {
    pub name: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub main: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
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
        let packages = NodeProvider::get_nix_packages(app, env)?;
        let mut setup_phase = SetupPhase::new(packages);
        if NodeProvider::uses_canvas(app) {
            setup_phase.add_libraries(vec!["libuuid".to_string(), "libGL".to_string()]);
        }
        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let install_cmd = NodeProvider::get_install_command(app);

        let mut install_phase = InstallPhase::new(install_cmd);

        // Package manage cache directories
        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager == "yarn" {
            install_phase.add_cache_directory(YARN_CACHE_DIR.to_string());
        } else if package_manager == "pnpm" {
            install_phase.add_cache_directory(PNPM_CACHE_DIR.to_string());
        } else if package_manager == "bun" {
            install_phase.add_cache_directory(BUN_CACHE_DIR.to_string());
        } else {
            install_phase.add_cache_directory(NPM_CACHE_DIR.to_string());
        }

        let all_deps = NodeProvider::get_all_deps(app)?;

        // Cypress cache directory
        if all_deps.get("cypress").is_some() {
            install_phase.add_cache_directory(CYPRESS_CACHE_DIR.to_string());
        }

        Ok(Some(install_phase))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let mut build_phase = BuildPhase::default();

        if NodeProvider::has_script(app, "build")? {
            let pkg_manager = NodeProvider::get_package_manager(app);
            build_phase.add_cmd(format!("{} run build", pkg_manager));
        }

        // Next build cache directories
        let next_cache_dirs = NodeProvider::find_next_packages(app)?;
        for dir in next_cache_dirs {
            let next_cache_dir = ".next/cache";
            build_phase.add_cache_directory(if dir.is_empty() {
                next_cache_dir.to_string()
            } else {
                format!("{}/{}", dir, next_cache_dir)
            });
        }

        // Node modules cache directory
        build_phase.add_cache_directory(NODE_MODULES_CACHE_DIR.to_string());

        Ok(Some(build_phase))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = NodeProvider::get_start_cmd(app)? {
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
        Ok(Some(NodeProvider::get_node_environment_variables()))
    }
}

impl NodeProvider {
    pub fn get_node_environment_variables() -> EnvironmentVariables {
        EnvironmentVariables::from([
            ("NODE_ENV".to_string(), "production".to_string()),
            ("NPM_CONFIG_PRODUCTION".to_string(), "false".to_string()),
            ("CI".to_string(), "true".to_string()),
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
        let package_manager = NodeProvider::get_package_manager(app);
        if NodeProvider::has_script(app, "start")? {
            return Ok(Some(format!("{} run start", package_manager)));
        }

        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                if package_manager == "bun" {
                    return Ok(Some(format!("bun {}", main)));
                } else {
                    return Ok(Some(format!("node {}", main)));
                }
            }
        }

        if app.includes_file("index.js") {
            if package_manager == "bun" {
                return Ok(Some("bun index.js".to_string()));
            } else {
                return Ok(Some("node index.js".to_string()));
            }
        } else if app.includes_file("index.ts") && package_manager == "bun" {
            return Ok(Some("bun index.ts".to_string()));
        }

        Ok(None)
    }

    /// Parses the package.json engines field and returns a Nix package if available
    pub fn get_nix_node_pkg(package_json: &PackageJson, environment: &Environment) -> Result<Pkg> {
        let env_node_version = environment.get_config_variable("NODE_VERSION");

        let pkg_node_version = package_json
            .engines
            .clone()
            .and_then(|engines| engines.get("node").map(|v| v.to_owned()));

        let node_version = pkg_node_version.or(env_node_version);

        let node_version = match node_version {
            Some(node_version) => node_version,
            None => return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME)),
        };

        // Any version will work, use latest
        if node_version == "*" {
            return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME));
        }

        // Parse `18` or `18.x` into nodejs-18_x
        // This also supports 18.x.x, or any number in place of the x.
        let re = Regex::new(r"^(\d*)(?:\.?(?:\d*|[xX]?)?)(?:\.?(?:\d*|[xX]?)?)").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, node_version.clone()) {
            return Ok(Pkg::new(node_pkg.as_str()));
        }

        // Parse `>=14.10.3 <16` into nodejs-14_x
        let re = Regex::new(r"^>=(\d+)").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, node_version) {
            return Ok(Pkg::new(node_pkg.as_str()));
        }

        Ok(Pkg::new(DEFAULT_NODE_PKG_NAME))
    }

    pub fn get_package_manager(app: &App) -> String {
        let mut pkg_manager = "npm";
        if app.includes_file("pnpm-lock.yaml") {
            pkg_manager = "pnpm";
        } else if app.includes_file("yarn.lock") {
            pkg_manager = "yarn";
        } else if app.includes_file("bun.lockb") {
            pkg_manager = "bun";
        }
        pkg_manager.to_string()
    }

    pub fn get_install_command(app: &App) -> String {
        let mut install_cmd = "npm i";
        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager == "pnpm" {
            install_cmd = "pnpm i --frozen-lockfile";
        } else if package_manager == "yarn" {
            if app.includes_file(".yarnrc.yml") {
                install_cmd = "yarn set version berry && yarn install --check-cache";
            } else {
                install_cmd = "yarn install --frozen-lockfile";
            }
        } else if app.includes_file("package-lock.json") {
            install_cmd = "npm ci";
        } else if app.includes_file("bun.lockb") {
            install_cmd = "bun i --no-save";
        }
        install_cmd.to_string()
    }

    /// Returns the nodejs nix package and the appropriate package manager nix image.
    pub fn get_nix_packages(app: &App, env: &Environment) -> Result<Vec<Pkg>> {
        let package_json: PackageJson = app.read_json("package.json")?;
        let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, env)?;
        let pm_pkg: Pkg;
        let mut pkgs = Vec::<Pkg>::new();

        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager != "bun" {
            pkgs.push(node_pkg);
        }
        if package_manager == "pnpm" {
            let lockfile = app.read_file("pnpm-lock.yaml").unwrap_or_default();
            if lockfile.starts_with("lockfileVersion: 5.3") {
                pm_pkg = Pkg::new("pnpm-6_x");
            } else {
                pm_pkg = Pkg::new("pnpm-7_x");
            }
        } else if package_manager == "yarn" {
            pm_pkg = Pkg::new("yarn-1_x");
        } else if package_manager == "bun" {
            pm_pkg = Pkg::new("bun");
        } else {
            // npm
            let lockfile = app.read_file("package-lock.json").unwrap_or_default();
            if lockfile.contains("\"lockfileVersion\": 1") {
                pm_pkg = Pkg::new("npm-6_x");
            } else {
                pm_pkg = Pkg::new("npm-8_x");
            }
        };
        pkgs.push(pm_pkg.from_overlay(NODE_OVERLAY));

        Ok(pkgs)
    }

    pub fn uses_canvas(app: &App) -> bool {
        let package_json = app.read_file("package.json").unwrap_or_default();
        let lock_json = app.read_file("package-lock.json").unwrap_or_default();
        let yarn_lock = app.read_file("yarn.lock").unwrap_or_default();
        let pnpm_yaml = app.read_file("pnpm-lock.yaml").unwrap_or_default();
        package_json.contains("\"canvas\"")
            || lock_json.contains("/canvas/")
            || yarn_lock.contains("/canvas/")
            || pnpm_yaml.contains("/canvas/")
    }

    pub fn find_next_packages(app: &App) -> Result<Vec<String>> {
        // Find all package.json files
        let package_json_files = app.find_files("**/package.json")?;

        let mut cache_dirs: Vec<String> = vec![];

        // Find package.json files with a "next build" build script and cache the associated .next/cache directory
        for file in package_json_files {
            // Don't find package.json files that are in node_modules
            if file
                .as_path()
                .to_str()
                .unwrap_or_default()
                .contains("node_modules")
            {
                continue;
            }

            let json: PackageJson = app.read_json(file.to_str().unwrap())?;
            let deps = NodeProvider::get_deps_from_package_json(&json);
            if deps.contains("next") {
                let relative = app.strip_source_path(file.as_path())?;
                cache_dirs.push(relative.parent().unwrap().to_str().unwrap().to_string());
            }
        }

        Ok(cache_dirs)
    }

    /// Finds all dependencies (dev and non-dev) of all package.json files in the app.
    pub fn get_all_deps(app: &App) -> Result<HashSet<String>> {
        // Find all package.json files
        let package_json_files = app.find_files("**/package.json")?;

        let mut all_deps: HashSet<String> = HashSet::new();

        for file in package_json_files {
            if file
                .as_path()
                .to_str()
                .unwrap_or_default()
                .contains("node_modules")
            {
                continue;
            }

            let json: PackageJson = app.read_json(file.to_str().unwrap())?;

            all_deps.extend(NodeProvider::get_deps_from_package_json(&json));
        }

        Ok(all_deps)
    }

    pub fn get_deps_from_package_json(json: &PackageJson) -> HashSet<String> {
        let mut all_deps: HashSet<String> = HashSet::new();

        let deps = json
            .dependencies
            .clone()
            .map(|deps| deps.keys().map(|k| k.to_string()).collect::<Vec<String>>())
            .unwrap_or_default();

        let dev_deps = json
            .dev_dependencies
            .clone()
            .map(|dev_deps| {
                dev_deps
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        all_deps.extend(deps.into_iter());
        all_deps.extend(dev_deps.into_iter());

        all_deps
    }
}

fn version_number_to_pkg(version: &u32) -> String {
    if AVAILABLE_NODE_VERSIONS.contains(version) {
        format!("nodejs-{}_x", version)
    } else {
        DEFAULT_NODE_PKG_NAME.to_string()
    }
}

fn parse_regex_into_pkg(re: &Regex, node_version: String) -> Option<String> {
    let matches: Vec<_> = re.captures_iter(node_version.as_str()).collect();
    if let Some(captures) = matches.get(0) {
        match captures[1].parse::<u32>() {
            Ok(version) => return Some(version_number_to_pkg(&version)),
            Err(_e) => {}
        }
    }

    None
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;

    fn engines_node(version: &str) -> Option<HashMap<String, String>> {
        Some(HashMap::from([("node".to_string(), version.to_string())]))
    }

    #[test]
    fn test_no_engines() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    ..Default::default()
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
                    name: Some(String::default()),
                    engines: engines_node("*"),
                    ..Default::default()
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
                    name: Some(String::default()),
                    engines: engines_node("14"),
                    ..Default::default()
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
                    name: Some(String::default()),
                    engines: engines_node("18.x"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-18_x")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("14.X"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_advanced_engine_x() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("18.x.x"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-18_x")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("14.X.x"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        Ok(())
    }

    #[test]
    fn test_advanced_engine_number() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("18.4.2"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-18_x")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("14.8.x"),
                    ..Default::default()
                },
                &Environment::default()
            )?,
            Pkg::new("nodejs-14_x")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("14.x.8"),
                    ..Default::default()
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
                    name: Some(String::default()),
                    engines: engines_node(">=14.10.3 <16"),
                    ..Default::default()
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
                    name: Some(String::default()),
                    ..Default::default()
                },
                &Environment::new(BTreeMap::from([(
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
        // this test now defaults to lts
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: engines_node("15"),
                    ..Default::default()
                },
                &Environment::default()
            )
            .unwrap()
            .name,
            "nodejs"
        );

        Ok(())
    }

    #[test]
    fn test_find_next_pacakges() -> Result<()> {
        assert_eq!(
            NodeProvider::find_next_packages(&App::new("./examples/node-monorepo")?)?,
            vec!["packages/client".to_string()]
        );
        assert_eq!(
            NodeProvider::find_next_packages(&App::new(
                "./examples/node-monorepo/packages/client"
            )?)?,
            vec!["".to_string()]
        );

        Ok(())
    }
}
