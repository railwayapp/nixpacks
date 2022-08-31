use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use self::nx::ProjectJson;

use super::Provider;
use crate::{
    nixpacks::{
        app::App,
        environment::{Environment, EnvironmentVariables},
        nix::pkg::Pkg,
        plan::{
            phase::{Phase, StartPhase},
            BuildPlan,
        },
    },
    providers::node::nx::NxJson,
};
use anyhow::bail;
use anyhow::Result;
use path_slash::PathExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
mod nx;

pub const NODE_OVERLAY: &str = "https://github.com/railwayapp/nix-npm-overlay/archive/main.tar.gz";

const DEFAULT_NODE_PKG_NAME: &str = "nodejs-16_x";
const AVAILABLE_NODE_VERSIONS: &[u32] = &[14, 16, 18];

const YARN_CACHE_DIR: &str = "/usr/local/share/.cache/yarn/v6";
const PNPM_CACHE_DIR: &str = "/root/.cache/pnpm";
const NPM_CACHE_DIR: &str = "/root/.npm";
const BUN_CACHE_DIR: &str = "/root/.bun";
const CYPRESS_CACHE_DIR: &str = "/root/.cache/Cypress";
const NODE_MODULES_CACHE_DIR: &str = "node_modules/.cache";
const NX_APP_NAME_ENV_VAR: &str = "NX_APP_NAME";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PackageJson {
    pub name: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub main: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
}

pub struct NodeProvider {}

impl Provider for NodeProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        // Setup
        let mut setup = Phase::setup(Some(NodeProvider::get_nix_packages(app, env)?));

        if NodeProvider::uses_node_dependency(app, "puppeteer") {
            // https://gist.github.com/winuxue/cfef08e2f5fe9dfc16a1d67a4ad38a01
            setup.add_apt_pkgs(vec![
                "libnss3".to_string(),
                "libatk1.0-0".to_string(),
                "libatk-bridge2.0-0".to_string(),
                "libcups2".to_string(),
                "libgbm1".to_string(),
                "libasound2".to_string(),
                "libpangocairo-1.0-0".to_string(),
                "libxss1".to_string(),
                "libgtk-3-0".to_string(),
                "libxshmfence1".to_string(),
                "libglu1".to_string(),
            ]);
        } else if NodeProvider::uses_node_dependency(app, "canvas") {
            setup.add_pkgs_libs(vec!["libuuid".to_string(), "libGL".to_string()]);
        }

        // Install
        let mut install = Phase::install(Some(NodeProvider::get_install_command(app)));
        install.add_cache_directory(NodeProvider::get_package_manager_cache_dir(app));
        install.add_path("/app/node_modules/.bin".to_string());

        // Cypress cache directory
        let all_deps = NodeProvider::get_all_deps(app)?;
        if all_deps.get("cypress").is_some() {
            install.add_cache_directory((*CYPRESS_CACHE_DIR).to_string());
        }

        // Build
        let mut build = if NodeProvider::is_nx_monorepo(app) {
            let app_name = NodeProvider::get_nx_app_name(app, env)?.unwrap();
            Phase::build(Some(format!("npx nx run {}:build:production", app_name)))
        } else if NodeProvider::has_script(app, "build")? {
            let pkg_manager = NodeProvider::get_package_manager(app);
            Phase::build(Some(format!("{} run build", pkg_manager)))
        } else {
            Phase::build(None)
        };

        // Next build cache directories
        let next_cache_dirs = NodeProvider::find_next_packages(app)?;
        for dir in next_cache_dirs {
            let next_cache_dir = ".next/cache";
            build.add_cache_directory(if dir.is_empty() {
                next_cache_dir.to_string()
            } else {
                format!("{}/{}", dir, next_cache_dir)
            });
        }

        // Node modules cache directory
        build.add_cache_directory((*NODE_MODULES_CACHE_DIR).to_string());

        // Start
        let start = NodeProvider::get_start_cmd(app, env)?.map(StartPhase::new);

        let mut plan = BuildPlan::new(vec![setup, install, build], start);
        plan.add_variables(NodeProvider::get_node_environment_variables());

        Ok(Some(plan))
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

    pub fn get_start_cmd(app: &App, env: &Environment) -> Result<Option<String>> {
        if NodeProvider::is_nx_monorepo(app) {
            let app_name = NodeProvider::get_nx_app_name(app, env)?.unwrap();
            let output_path = NodeProvider::get_nx_output_path(app, env)?;
            let project_json = NodeProvider::get_nx_project_json_for_app(app, env)?;

            if let Some(start_target) = project_json.targets.start {
                if start_target.configurations.is_some()
                    && start_target.configurations.unwrap().production.is_some()
                {
                    return Ok(Some(format!("npx nx run {}:start:production ", app_name)));
                }
                return Ok(Some(format!("npx nx run {}:start", app_name)));
            }

            if project_json.targets.build.executor == "@nrwl/next:build" {
                return Ok(Some(format!("cd {} && npm run start", output_path)));
            }

            let main = project_json.targets.build.options.main;
            if let Some(main_path) = main {
                let current_path = PathBuf::from(main_path.as_str().unwrap());
                let file_name = current_path.file_stem().unwrap().to_str().unwrap();

                return Ok(Some(format!("node {}/{}.js", output_path, file_name)));
            }
            return Ok(Some(format!("node {}/index.js", output_path)));
        }

        let package_manager = NodeProvider::get_package_manager(app);
        if NodeProvider::has_script(app, "start")? {
            return Ok(Some(format!("{} run start", package_manager)));
        }

        let package_json: PackageJson = app.read_json("package.json")?;
        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                if package_manager == "bun" {
                    return Ok(Some(format!("bun {}", main)));
                }
                return Ok(Some(format!("node {}", main)));
            }
        }

        if app.includes_file("index.js") {
            if package_manager == "bun" {
                return Ok(Some("bun index.js".to_string()));
            }
            return Ok(Some("node index.js".to_string()));
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
            .and_then(|engines| engines.get("node").cloned());

        let node_version = env_node_version.or(pkg_node_version);

        let node_version = match node_version {
            Some(node_version) => node_version,
            None => return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME)),
        };

        // Any version will work, use latest
        if node_version == "*" {
            return Ok(Pkg::new(DEFAULT_NODE_PKG_NAME));
        }

        // This also supports 18.x.x, or any number in place of the x.
        let re = Regex::new(r"^(\d*)(?:\.?(?:\d*|[xX]?)?)(?:\.?(?:\d*|[xX]?)?)").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, &node_version) {
            return Ok(Pkg::new(node_pkg.as_str()));
        }

        // Parse `>=14.10.3 <16` into nodejs-14_x
        let re = Regex::new(r"^>=(\d+)").unwrap();
        if let Some(node_pkg) = parse_regex_into_pkg(&re, &node_version) {
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

    fn get_package_manager_cache_dir(app: &App) -> String {
        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager == "yarn" {
            (*YARN_CACHE_DIR).to_string()
        } else if package_manager == "pnpm" {
            (*PNPM_CACHE_DIR).to_string()
        } else if package_manager == "bun" {
            (*BUN_CACHE_DIR).to_string()
        } else {
            (*NPM_CACHE_DIR).to_string()
        }
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

    pub fn uses_node_dependency(app: &App, dependency: &str) -> bool {
        [
            "package.json",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
        ]
        .iter()
        .any(|file| app.read_file(file).unwrap_or_default().contains(dependency))
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
                cache_dirs.push(relative.parent().unwrap().to_slash().unwrap().into_owned());
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
            .map(|deps| deps.keys().cloned().collect::<Vec<String>>())
            .unwrap_or_default();

        let dev_deps = json
            .dev_dependencies
            .clone()
            .map(|dev_deps| dev_deps.keys().cloned().collect::<Vec<String>>())
            .unwrap_or_default();

        all_deps.extend(deps.into_iter());
        all_deps.extend(dev_deps.into_iter());

        all_deps
    }

    pub fn is_nx_monorepo(app: &App) -> bool {
        app.includes_file("nx.json")
    }

    pub fn get_nx_app_name(app: &App, env: &Environment) -> Result<Option<String>> {
        if let Some(app_name) = env.get_config_variable(NX_APP_NAME_ENV_VAR) {
            return Ok(Some(app_name));
        }

        if let Ok(nx_json) = app.read_json::<NxJson>("nx.json") {
            if let Some(default_project) = nx_json.default_project {
                return Ok(Some(default_project.as_str().unwrap().to_owned()));
            }
        }

        bail!("Could not derive nx app to build and run. Please add a default project to your nx config or set NIXPACKS_{}", NX_APP_NAME_ENV_VAR);
    }

    pub fn get_nx_project_json_for_app(app: &App, env: &Environment) -> Result<ProjectJson> {
        let app_name = NodeProvider::get_nx_app_name(app, env)?.unwrap();
        let project_path = format!("./apps/{}/project.json", app_name);
        app.read_json::<ProjectJson>(&project_path)
    }

    pub fn get_nx_output_path(app: &App, env: &Environment) -> Result<String> {
        let project_json = NodeProvider::get_nx_project_json_for_app(app, env)?;
        if let Some(output_path) = project_json.targets.build.options.output_path {
            if let Some(output_path) = output_path.as_str() {
                return Ok(output_path.to_string());
            }
        }

        if let Ok(Some(app_name)) = NodeProvider::get_nx_app_name(app, env) {
            return Ok(format!("dist/apps/{}", app_name));
        };

        bail!("Could not derive nx output path. Please add an output_path to your project.json");
    }
}

fn version_number_to_pkg(version: u32) -> String {
    if AVAILABLE_NODE_VERSIONS.contains(&version) {
        format!("nodejs-{}_x", version)
    } else {
        DEFAULT_NODE_PKG_NAME.to_string()
    }
}

fn parse_regex_into_pkg(re: &Regex, node_version: &str) -> Option<String> {
    let matches: Vec<_> = re.captures_iter(node_version).collect();
    if let Some(captures) = matches.get(0) {
        match captures[1].parse::<u32>() {
            Ok(version) => return Some(version_number_to_pkg(version)),
            Err(_e) => {}
        }
    }

    None
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;

    fn engines_node(version: &str) -> HashMap<String, String> {
        HashMap::from([("node".to_string(), version.to_string())])
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
                    engines: Some(engines_node("*")),
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
                    engines: Some(engines_node("14")),
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
                    engines: Some(engines_node("18.x")),
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
                    engines: Some(engines_node("14.X")),
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
                    engines: Some(engines_node("18.x.x")),
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
                    engines: Some(engines_node("14.X.x")),
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
                    engines: Some(engines_node("18.4.2")),
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
                    engines: Some(engines_node("14.8.x")),
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
                    engines: Some(engines_node("14.x.8")),
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
                    engines: Some(engines_node(">=14.10.3 <16")),
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
                    engines: Some(engines_node("15")),
                    ..Default::default()
                },
                &Environment::default()
            )?
            .name,
            DEFAULT_NODE_PKG_NAME
        );

        Ok(())
    }

    #[test]
    fn test_find_next_packages() -> Result<()> {
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
