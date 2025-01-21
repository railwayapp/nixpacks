use self::{moon::Moon, nx::Nx, spa::SpaProvider, turborepo::Turborepo};
use super::Provider;
use crate::nixpacks::plan::merge::Mergeable;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::Result;
use node_semver::Range;
use path_slash::PathExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

mod moon;
mod nx;
mod spa;
mod turborepo;

pub const NODE_OVERLAY: &str = "https://github.com/railwayapp/nix-npm-overlay/archive/main.tar.gz";

const NODE_NIXPKGS_ARCHIVE: &str = "5624e1334b26ddc18da37e132b6fa8e93b481468";

// We need to use a specific commit hash for Node versions <16 since it is EOL in the latest Nix packages
const NODE_LT_16_ARCHIVE: &str = "bf744fe90419885eefced41b3e5ae442d732712d";

const DEFAULT_NODE_VERSION: u32 = 18;
const AVAILABLE_NODE_VERSIONS: &[u32] = &[14, 16, 18, 20, 22, 23];

const YARN_CACHE_DIR: &str = "/usr/local/share/.cache/yarn/v6";
const PNPM_CACHE_DIR: &str = "/root/.local/share/pnpm/store/v3";
const NPM_CACHE_DIR: &str = "/root/.npm";
const BUN_CACHE_DIR: &str = "/root/.bun";
const CYPRESS_CACHE_DIR: &str = "/root/.cache/Cypress";
const NODE_MODULES_CACHE_DIR: &str = "node_modules/.cache";

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct TsConfigJson {
    #[serde(rename = "compilerOptions")]
    compiler_options: Option<TsConfigCompilerOptions>,
    extends: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct TsConfigCompilerOptions {
    incremental: Option<bool>,
    #[serde(rename = "tsBuildInfoFile")]
    ts_build_info_file: Option<String>,
    #[serde(rename = "outDir")]
    out_dir: Option<String>,
}

impl Mergeable for TsConfigCompilerOptions {
    fn merge(
        c1: &TsConfigCompilerOptions,
        c2: &TsConfigCompilerOptions,
    ) -> TsConfigCompilerOptions {
        let mut new_compileroptions = c1.clone();
        let compileroptions2 = c2.clone();
        new_compileroptions.incremental = compileroptions2
            .incremental
            .or(new_compileroptions.incremental);
        new_compileroptions.out_dir = compileroptions2.out_dir.or(new_compileroptions.out_dir);
        new_compileroptions.ts_build_info_file = compileroptions2
            .ts_build_info_file
            .or(new_compileroptions.ts_build_info_file);
        new_compileroptions
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Workspaces {
    Array(Vec<String>),
    Unknown(Value),
}

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

    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,

    pub workspaces: Option<Workspaces>,

    #[serde(rename = "cacheDirectories")]
    pub cache_directories: Option<Vec<String>>,
}

impl PackageJson {
    /// searches dependencies and dev_dependencies in package.json for a given dependency
    fn has_dependency(&self, dep: &str) -> bool {
        if let Some(deps) = &self.dependencies {
            if deps.contains_key(dep) {
                return true;
            }
        } else if let Some(deps) = &self.dev_dependencies {
            if deps.contains_key(dep) {
                return true;
            }
        }
        false
    }
}

#[derive(Default, Debug)]
pub struct NodeProvider {}

impl Provider for NodeProvider {
    fn name(&self) -> &'static str {
        "node"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.json"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        // Setup
        let mut setup = Phase::setup(Some(NodeProvider::get_nix_packages(app, env)?));
        setup.set_nix_archive(NodeProvider::get_nix_archive(app)?);
        if NodeProvider::uses_node_dependency(app, "prisma") {
            setup.add_nix_pkgs(&[Pkg::new("openssl")]);
        }

        if NodeProvider::uses_node_dependency(app, "sharp") {
            setup.add_pkgs_libs(vec!["gcc-unwrapped".to_string()]);
        }

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
        }

        if NodeProvider::uses_node_dependency(app, "canvas") {
            setup.add_pkgs_libs(vec!["libuuid".to_string(), "libGL".to_string()]);
        }

        // Install
        let corepack = NodeProvider::uses_corepack(app, env)?;
        let mut install = Phase::install(if corepack {
            Some("npm install -g corepack@0.24.1 && corepack enable".to_string())
        } else {
            NodeProvider::get_install_command(app)
        });

        if corepack {
            let install_cmd = NodeProvider::get_install_command(app);

            if install_cmd.is_some() {
                install.add_cmd(install_cmd.unwrap_or_default());
            }
        }

        install.add_cache_directory(NodeProvider::get_package_manager_cache_dir(app));
        install.add_path("/app/node_modules/.bin".to_string());

        // Cypress cache directory
        let all_deps = NodeProvider::get_all_deps(app)?;
        if all_deps.contains("cypress") {
            install.add_cache_directory((*CYPRESS_CACHE_DIR).to_string());
        }

        // Build
        let mut build = Phase::build(NodeProvider::get_build_cmd(app, env)?);

        // Next build cache directories
        let next_cache_dirs = NodeProvider::find_next_packages(app)?;
        for dir in next_cache_dirs {
            let next_cache_dir = ".next/cache";
            build.add_cache_directory(if dir.is_empty() {
                next_cache_dir.to_string()
            } else {
                format!("{dir}/{next_cache_dir}")
            });
        }

        // Node modules cache directory
        build.add_cache_directory((*NODE_MODULES_CACHE_DIR).to_string());
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();
        if let Some(cache_directories) = package_json.cache_directories {
            for dir in cache_directories {
                build.add_cache_directory(dir);
            }
        }

        NodeProvider::cache_tsbuildinfo_file(app, &mut build);

        if Moon::is_moon_repo(app, env) {
            build.add_cache_directory(".moon/cache/outputs");
        }

        // Start
        let start = NodeProvider::get_start_cmd(app, env)?.map(StartPhase::new);

        let mut phases = vec![setup, install, build];
        if let Some(caddy) = SpaProvider::caddy_phase(app, env) {
            phases.push(caddy);
        }
        let is_spa = SpaProvider::is_spa(app);

        let mut plan = BuildPlan::new(&phases, start);
        if SpaProvider::caddy_phase(app, env).is_some() {
            plan.add_static_assets(SpaProvider::static_assets());
        }
        plan.add_variables(NodeProvider::get_node_environment_variables());
        if is_spa {
            plan.add_variables(EnvironmentVariables::from([(
                "NIXPACKS_SPA_OUTPUT_DIR".to_string(),
                env.get_config_variable("SPA_OUT_DIR")
                    .unwrap_or(SpaProvider::get_output_directory(app)),
            )]));
        }
        Ok(Some(plan))
    }
}

impl NodeProvider {
    pub fn get_node_environment_variables() -> EnvironmentVariables {
        EnvironmentVariables::from([
            ("NODE_ENV".to_string(), "production".to_string()),
            ("NPM_CONFIG_PRODUCTION".to_string(), "false".to_string()),
            // CI required for various node tooling
            ("CI".to_string(), "true".to_string()),
        ])
    }

    pub fn has_script(app: &App, script: &str) -> Result<bool> {
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();
        if let Some(scripts) = package_json.scripts {
            if scripts.contains_key(script) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn uses_corepack(app: &App, env: &Environment) -> Result<bool> {
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();
        let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, app, env)?;

        // Corepack is not supported for Node 14
        if node_pkg.name.contains("14") {
            return Ok(false);
        }

        if let Some(package_manager) = package_json.package_manager {
            if package_manager.starts_with("npm") {
                // Fall back to just using the system npm version.
                return Ok(false);
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_build_cmd(app: &App, env: &Environment) -> Result<Option<String>> {
        if Moon::is_moon_repo(app, env) {
            return Ok(Some(Moon::get_build_cmd(app, env)));
        }

        if Nx::is_nx_monorepo(app, env) {
            if let Some(nx_build_cmd) = Nx::get_nx_build_cmd(app, env) {
                return Ok(Some(nx_build_cmd));
            }
        }

        if Turborepo::is_turborepo(app) {
            if let Ok(Some(turbo_build_cmd)) = Turborepo::get_actual_build_cmd(app, env) {
                return Ok(Some(turbo_build_cmd));
            }
        }

        if NodeProvider::has_script(app, "build")? {
            let pkg_manager = NodeProvider::get_package_manager(app);
            Ok(Some(format!("{pkg_manager} run build")))
        } else {
            Ok(None)
        }
    }

    pub fn get_start_cmd(app: &App, env: &Environment) -> Result<Option<String>> {
        let executor = NodeProvider::get_executor(app);
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();

        if Moon::is_moon_repo(app, env) {
            return Ok(Some(Moon::get_start_cmd(app, env)));
        }

        if Nx::is_nx_monorepo(app, env) {
            if let Some(nx_start_cmd) = Nx::get_nx_start_cmd(app, env)? {
                return Ok(Some(nx_start_cmd));
            }
        }

        if Turborepo::is_turborepo(app) {
            if let Ok(Some(turbo_start_cmd)) =
                Turborepo::get_actual_start_cmd(app, env, &package_json)
            {
                return Ok(Some(turbo_start_cmd));
            }
        }

        if let Some(start) = SpaProvider::start_command(app, env) {
            return Ok(Some(start));
        }

        let package_manager = NodeProvider::get_package_manager(app);
        if NodeProvider::has_script(app, "start")? {
            return Ok(Some(format!("{package_manager} run start")));
        }

        if let Some(main) = package_json.main {
            if app.includes_file(&main) {
                return Ok(Some(format!("{executor} {main}")));
            }
        }

        if app.includes_file("index.js") {
            return Ok(Some(format!("{executor} index.js")));
        } else if app.includes_file("index.ts") && package_manager == "bun" {
            return Ok(Some("bun index.ts".to_string()));
        }

        Ok(None)
    }

    /// Parses the package.json engines field and returns a Nix package if available
    pub fn get_nix_node_pkg(
        package_json: &PackageJson,
        app: &App,
        environment: &Environment,
    ) -> Result<Pkg> {
        let default_node_pkg_name = version_number_to_pkg(DEFAULT_NODE_VERSION);
        let env_node_version = environment.get_config_variable("NODE_VERSION");

        let pkg_node_version = package_json
            .engines
            .clone()
            .and_then(|engines| engines.get("node").cloned());

        let nvmrc_node_version = if app.includes_file(".nvmrc") {
            let nvmrc = app.read_file(".nvmrc")?;
            Some(parse_nvmrc(&nvmrc))
        } else {
            None
        };

        let dot_node_version = if app.includes_file(".node-version") {
            let node_version_file = app.read_file(".node-version")?;
            // Using simple string transform since .node-version don't currently have a convention around the use of lts/* implemented in parse_nvmrc method
            Some(node_version_file.trim().replace('v', ""))
        } else {
            None
        };

        let node_version = env_node_version
            .or(pkg_node_version)
            .or(nvmrc_node_version)
            .or(dot_node_version);

        let node_version = match node_version {
            Some(node_version) => node_version,
            None => return Ok(Pkg::new(default_node_pkg_name.as_str())),
        };

        // Any version will work, use default
        if node_version == "*" {
            return Ok(Pkg::new(default_node_pkg_name.as_str()));
        }

        let node_pkg = parse_node_version_into_pkg(&node_version);
        Ok(Pkg::new(node_pkg.as_str()))
    }

    pub fn get_package_manager(app: &App) -> String {
        // Checks for the package manager in root's package.json
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();

        // Attempt to identify the package manager from `package.json`
        if let Some(pkg) = package_json
            .package_manager
            .as_deref()
            .and_then(|p| p.split('@').next())
        {
            if matches!(pkg, "npm" | "pnpm" | "yarn" | "bun") {
                return pkg.to_string();
            }
        }

        // Check for lockfiles to infer the package manager
        if app.includes_file("pnpm-lock.yaml") {
            return "pnpm".to_string();
        }

        if app.includes_file("yarn.lock") {
            return "yarn".to_string();
        }

        if app.includes_file("bun.lockb") || app.includes_file("bun.lock") {
            return "bun".to_string();
        }

        // fallbacks to npm
        "npm".to_string()
    }

    pub fn get_package_manager_dlx_command(app: &App) -> String {
        let pkg_manager = NodeProvider::get_package_manager(app);
        match pkg_manager.as_str() {
            "pnpm" => "pnpx",
            "yarn" => "yarn",
            _ => "npx",
        }
        .to_string()
    }

    pub fn get_install_command(app: &App) -> Option<String> {
        if !app.includes_file("package.json") {
            return None;
        }

        let mut install_cmd = "npm i".to_string();
        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager == "pnpm" {
            install_cmd = "pnpm i --frozen-lockfile".to_string();
        } else if package_manager == "yarn" {
            // TODO: When using Corepack and modern Yarn, we may not have a .yarnrc.yml - need to
            //       read the Yarn version from stdout after enabling Corepack.
            if app.includes_file(".yarnrc.yml") {
                install_cmd = "yarn install --check-cache".to_string();
            } else {
                install_cmd = "yarn install --frozen-lockfile".to_string();
            }
        } else if app.includes_file("package-lock.json") {
            install_cmd = "npm ci".to_string();
        } else if app.includes_file("bun.lockb") || app.includes_file("bun.lock") {
            install_cmd = "bun i --no-save".to_string();
        }

        Some(install_cmd)
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

    fn get_executor(app: &App) -> String {
        let package_manager = NodeProvider::get_package_manager(app);
        if package_manager == *"bun" {
            "bun"
        } else {
            "node"
        }
        .to_string()
    }

    /// Returns the Nix archive to use for the Node and related packages
    pub fn get_nix_archive(app: &App) -> Result<String> {
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();
        let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, app, &Environment::default())?;
        let uses_le_16 = node_pkg.name.contains("14") || node_pkg.name.contains("16");

        if uses_le_16 {
            Ok(NODE_LT_16_ARCHIVE.to_string())
        } else {
            Ok(NODE_NIXPKGS_ARCHIVE.to_string())
        }
    }

    /// Returns the nodejs nix package and the appropriate package manager nix image.
    pub fn get_nix_packages(app: &App, env: &Environment) -> Result<Vec<Pkg>> {
        let package_json: PackageJson = if app.includes_file("package.json") {
            app.read_json("package.json")?
        } else {
            PackageJson::default()
        };
        let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, app, env)?;

        let pm_pkg: Pkg;
        let mut pkgs = Vec::<Pkg>::new();

        let package_manager = NodeProvider::get_package_manager(app);
        pkgs.push(node_pkg);

        if package_manager == "pnpm" {
            let lockfile = app.read_file("pnpm-lock.yaml").unwrap_or_default();
            if lockfile.starts_with("lockfileVersion: 5.3") {
                pm_pkg = Pkg::new("pnpm-6_x");
            } else if lockfile.starts_with("lockfileVersion: 5.4") {
                pm_pkg = Pkg::new("pnpm-7_x");
            } else if lockfile.starts_with("lockfileVersion: '6.0'") {
                pm_pkg = Pkg::new("pnpm-8_x");
            } else {
                // Default to pnpm 9
                pm_pkg = Pkg::new("pnpm-9_x");
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
            } else if lockfile.contains("\"lockfileVersion\": 2") {
                pm_pkg = Pkg::new("npm-8_x");
            } else {
                // npm v9 uses lockfile v3 as default
                pm_pkg = Pkg::new("npm-9_x");
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

        all_deps.extend(deps);
        all_deps.extend(dev_deps);

        all_deps
    }

    pub fn cache_tsbuildinfo_file(app: &App, build: &mut Phase) {
        let mut ts_config: TsConfigJson = app.read_json("tsconfig.json").unwrap_or_default();
        if let Some(ref extends) = ts_config.extends {
            let ex: TsConfigJson = app.read_json(extends.as_str()).unwrap_or_default();
            ts_config.compiler_options = Some(TsConfigCompilerOptions::merge(
                &ex.compiler_options.unwrap_or_default(),
                &ts_config.compiler_options.unwrap_or_default(),
            ));
        }

        if let Some(compiler_options) = ts_config.compiler_options {
            if let Some(incremental) = compiler_options.incremental {
                // if incremental is enabled
                if incremental {
                    let tsbuildinfo =
                        if let Some(ts_build_info_file) = compiler_options.ts_build_info_file {
                            // if config file is explicitly provided
                            ts_build_info_file
                        } else if let Some(out_dir) = compiler_options.out_dir {
                            // if it is not provided but outdir is, use that
                            format!("{out_dir}/tsconfig.tsbuildinfo")
                        } else {
                            // if not out dir is set
                            "tsconfig.tsbuildinfo".to_string()
                        };

                    if app.includes_file(tsbuildinfo.as_str()) {
                        build.add_cache_directory(tsbuildinfo);
                    }
                }
            }
        };
    }
}

fn version_number_to_pkg(version: u32) -> String {
    if AVAILABLE_NODE_VERSIONS.contains(&version) {
        format!("nodejs_{version}")
    } else {
        format!("nodejs_{DEFAULT_NODE_VERSION}")
    }
}

fn parse_node_version_into_pkg(node_version: &str) -> String {
    let default_node_pkg_name = version_number_to_pkg(DEFAULT_NODE_VERSION);
    let range: Range = node_version.parse().unwrap_or_else(|_| {
        eprintln!("Warning: node version {node_version} is not valid, using default node version {default_node_pkg_name}");
        Range::parse(DEFAULT_NODE_VERSION.to_string()).unwrap()
    });
    let mut available_lts_node_versions = AVAILABLE_NODE_VERSIONS
        .iter()
        .filter(|v| *v % 2 == 0)
        .collect::<Vec<_>>();

    // use newest node version first
    available_lts_node_versions.sort_by(|a, b| b.cmp(a));
    for version_number in available_lts_node_versions {
        let version_range_string = format!("{version_number}.x.x");
        let version_range: Range = version_range_string.parse().unwrap();
        if version_range.allows_any(&range) {
            return version_number_to_pkg(*version_number);
        }
    }
    default_node_pkg_name
}

fn parse_nvmrc(nvmrc_content: &str) -> String {
    let lts_versions: HashMap<&str, u32> = {
        let mut nvm_map = HashMap::new();
        nvm_map.insert("lts/*", 22);
        nvm_map.insert("lts/jod", 22);
        nvm_map.insert("lts/argon", 4);
        nvm_map.insert("lts/boron", 6);
        nvm_map.insert("lts/carbon", 8);
        nvm_map.insert("lts/dubnium", 10);
        nvm_map.insert("lts/erbium", 12);
        nvm_map.insert("lts/fermium", 14);
        nvm_map.insert("lts/gallium", 16);
        nvm_map.insert("lts/hydrogen", 18);
        nvm_map.insert("lts/iron", 20);
        nvm_map
    };

    let trimmed_version = nvmrc_content.trim();
    if let Some(&version) = lts_versions.get(trimmed_version) {
        return version.to_string();
    }

    // Only remove v if it is in the starting character, lts/ will never have that in starting
    trimmed_version
        .strip_prefix('v')
        .unwrap_or(trimmed_version)
        .to_string()
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new(version_number_to_pkg(DEFAULT_NODE_VERSION).as_str())
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new(version_number_to_pkg(DEFAULT_NODE_VERSION).as_str())
        );

        Ok(())
    }

    #[test]
    fn test_latest_lts_version() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node(">=18")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new(version_number_to_pkg(22).as_str())
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("14.X")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("14.X.x")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("14.8.x")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
        );

        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("14.x.8")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
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
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
        );

        Ok(())
    }

    #[test]
    fn test_engine_caret_range() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("^14.10.3")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
        );

        Ok(())
    }

    #[test]
    fn test_engine_multi_range() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("1.2.3 || 14.10.3")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
        );

        Ok(())
    }

    #[test]
    fn test_engine_multi_satisfied_range() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("14.10.3 || 18.10.0")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
        );

        Ok(())
    }

    #[test]
    fn test_invalid_node_version() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    engines: Some(engines_node("abc")),
                    ..Default::default()
                },
                &App::new("examples/node")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
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
                &App::new("examples/node")?,
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_NODE_VERSION".to_string(),
                    "14".to_string()
                )]))
            )?,
            Pkg::new("nodejs_14")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_nvmrc() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    ..Default::default()
                },
                &App::new("examples/node-nvmrc")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_14")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_node_version_file() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    ..Default::default()
                },
                &App::new("examples/node-node-version")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_22")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_nvmrc_lts() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    ..Default::default()
                },
                &App::new("examples/node-nvmrc-lts")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_20")
        );

        Ok(())
    }

    #[test]
    fn test_invalid_version_from_nvmrc_lts() -> Result<()> {
        assert_eq!(
            NodeProvider::get_nix_node_pkg(
                &PackageJson {
                    name: Some(String::default()),
                    ..Default::default()
                },
                &App::new("examples/node-nvmrc-invalid-lts")?,
                &Environment::default()
            )?,
            Pkg::new("nodejs_18")
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
                &App::new("examples/node")?,
                &Environment::default()
            )?
            .name,
            version_number_to_pkg(DEFAULT_NODE_VERSION).as_str()
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
            vec![String::new()]
        );

        Ok(())
    }

    #[test]
    fn test_correct_package_manager_monorepo_root() -> Result<()> {
        assert_eq!(
            NodeProvider::get_package_manager(&App::new("examples/node-pnpm-monorepo")?),
            "pnpm"
        );

        Ok(())
    }

    #[test]
    fn test_correct_package_manager_monorepo_subpkg() -> Result<()> {
        assert_eq!(
            NodeProvider::get_package_manager(&App::new("examples/node-pnpm-monorepo/apps/docs")?),
            "pnpm"
        );

        Ok(())
    }
}
