use std::env::consts::ARCH;
use std::fmt::Write as _;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Result};
use cargo_toml::{Manifest, Workspace};
use regex::Regex;

const RUST_OVERLAY: &str = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
const DEFAULT_RUST_PACKAGE: &str = "rust-bin.stable.latest.default";

const CARGO_GIT_CACHE_DIR: &str = "/root/.cargo/git";
const CARGO_REGISTRY_CACHE_DIR: &str = "/root/.cargo/registry";
const CARGO_TARGET_CACHE_DIR: &str = "target";

const NIX_ARCHIVE: &str = "ef56e777fedaa4da8c66a150081523c5de1e0171";

pub struct RustProvider {}

impl Provider for RustProvider {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Cargo.toml"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = RustProvider::get_setup(app, env)?;
        let build = RustProvider::get_build(app, env)?;
        let start = RustProvider::get_start(app, env)?;

        let mut plan = BuildPlan::new(&vec![setup, build], start);
        plan.add_variables(EnvironmentVariables::from([(
            "ROCKET_ADDRESS".to_string(),
            "0.0.0.0".to_string(),
        )]));

        Ok(Some(plan))
    }
}

impl RustProvider {
    pub(crate) fn get_setup(app: &App, env: &Environment) -> Result<Phase> {
        let mut rust_pkg: Pkg = RustProvider::get_rust_pkg(app, env)?;

        if let Some(target) = RustProvider::get_target(app, env)? {
            rust_pkg = rust_pkg.set_override("targets", &format!("[\"{target}\"]"));
        }

        let mut setup = Phase::setup(Some(vec![
            Pkg::new("binutils"),
            Pkg::new("gcc"),
            rust_pkg.from_overlay(RUST_OVERLAY),
        ]));

        // Include the rust toolchain file so we can install that rust version with Nix
        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app) {
            setup.add_file_dependency(toolchain_file);
        }

        // Custom libs for openssl
        if RustProvider::uses_openssl(app)? {
            setup.add_pkgs_libs(vec!["openssl".to_string(), "openssl.dev".to_string()]);
        }

        if RustProvider::should_use_musl(app, env)? {
            setup.add_nix_pkgs(&[Pkg::new("musl"), Pkg::new("musl.dev")]);
        }

        setup.set_nix_archive(NIX_ARCHIVE.to_string());

        Ok(setup)
    }

    pub(crate) fn get_build(app: &App, env: &Environment) -> Result<Phase> {
        let mut build = Phase::build(None);
        if !app.includes_file("Cargo.toml") {
            return Ok(build);
        }

        build.add_cmd("mkdir -p bin");
        build.depends_on = Some(vec!["setup".to_string()]);

        let mut build_cmd = "cargo build --release".to_string();

        // Default binary suffix (.wasm || none)
        let bin_suffix = RustProvider::get_bin_suffix(app, env, None);

        if let Some(target) = RustProvider::get_target(app, env)? {
            if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
                write!(build_cmd, " --package {workspace} --target {target}")?;

                build.add_cmd(build_cmd);
                build.add_cmd(format!(
                    "cp target/{target}/release/{workspace}{bin_suffix} bin"
                ));
            } else if let Some(bins) = RustProvider::get_bins(app)? {
                write!(build_cmd, " --target {target}")?;

                build.add_cmd(build_cmd);

                for bin in bins {
                    build.add_cmd(format!("cp target/{target}/release/{bin}{bin_suffix} bin"));
                }
            }
        } else if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
            write!(build_cmd, " --package {workspace}")?;
            build.add_cmd(build_cmd);
            build.add_cmd(format!("cp target/release/{workspace}{bin_suffix} bin"));
        } else if let Some(bins) = RustProvider::get_bins(app)? {
            build.add_cmd(build_cmd);

            for bin in bins {
                build.add_cmd(format!("cp target/release/{bin}{bin_suffix} bin"));
            }
        }

        build.add_cache_directory(CARGO_GIT_CACHE_DIR.to_string());
        build.add_cache_directory(CARGO_REGISTRY_CACHE_DIR.to_string());

        if RustProvider::get_app_name(app)?.is_some() {
            // Cache target directory
            build.add_cache_directory(CARGO_TARGET_CACHE_DIR.to_string());
        }

        Ok(build)
    }

    fn get_bin_suffix(app: &App, env: &Environment, _: Option<String>) -> String {
        // wasm32-wasi binaries are created with .wasm
        if RustProvider::should_make_wasm32_wasi(app, env) {
            ".wasm"
        } else {
            ""
        }
        .into()
    }

    fn get_bins(app: &App) -> Result<Option<Vec<String>>> {
        let mut bins = vec![];

        // Support the main bin
        if let Some(name) = RustProvider::get_app_name(app)? {
            if app.includes_file("src/main.rs") {
                bins.push(name);
            }
        }

        if app.includes_directory("src/bin") {
            let find_bins = app.find_files("src/bin/*")?;

            for bin in find_bins {
                let bin_name = bin
                    .file_name()
                    .context("Could not get file name for bin")?
                    .to_str()
                    .context("Could not convert bin name to string")?
                    .split('.')
                    .collect::<Vec<_>>();

                let bin_name = bin_name[0..bin_name.len() - 1].join(".");

                bins.push(bin_name);
            }
        }

        if bins.is_empty() {
            return Ok(None);
        }

        Ok(Some(bins))
    }

    pub(crate) fn get_start(app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        if (RustProvider::get_target(app, env)?).is_some() {
            if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
                let mut start = StartPhase::new(format!("./bin/{workspace}"));
                start.run_in_slim_image();
                start.add_file_dependency(format!("./bin/{workspace}"));

                Ok(Some(start))
            } else if let Some(bin) = RustProvider::get_start_bin(app, env)? {
                let mut start = StartPhase::new(bin.clone());
                start.run_in_slim_image();
                start.add_file_dependency(bin);
                Ok(Some(start))
            } else {
                Ok(None)
            }
        } else if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
            Ok(Some(StartPhase::new(format!("./bin/{workspace}"))))
        } else if let Some(bin) = RustProvider::get_start_bin(app, env)? {
            Ok(Some(StartPhase::new(bin)))
        } else {
            Ok(None)
        }
    }

    fn get_app_name(app: &App) -> Result<Option<String>> {
        if let Some(toml_file) = RustProvider::parse_cargo_toml(app)? {
            if let Some(package) = toml_file.package {
                let name = package.name;
                return Ok(Some(name));
            }
        }

        Ok(None)
    }

    fn get_start_bin(app: &App, env: &Environment) -> Result<Option<String>> {
        if let Some(bins) = RustProvider::get_bins(app)? {
            let mut bin: Option<String> = None;

            if bins.len() == 1 {
                bin = Some(bins[0].clone());
            } else if let Some(env_bin_name) = env.get_config_variable("RUST_BIN") {
                let found_bin = bins
                    .into_iter()
                    .find(|bin| bin == &env_bin_name)
                    .context(format!("Could not find binary named {env_bin_name}"))?;

                bin = Some(found_bin);
            } else if let Some(found_bin) = RustProvider::parse_cargo_toml(app)?
                .and_then(|manifest| manifest.package)
                .and_then(|package| package.default_run)
            {
                bin = Some(found_bin);
            }

            let bin_suffix = RustProvider::get_bin_suffix(app, env, None);

            if let Some(bin) = bin {
                Ok(Some(format!("./bin/{bin}{bin_suffix}")))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn get_target(app: &App, env: &Environment) -> Result<Option<String>> {
        // Target may be defined in .config/cargo.toml
        if RustProvider::should_make_wasm32_wasi(app, env) {
            Ok(Some("wasm32-wasi".into()))
        } else if RustProvider::should_use_musl(app, env)? {
            Ok(Some(format!("{ARCH}-unknown-linux-musl")))
        } else {
            Ok(None)
        }
    }

    fn parse_cargo_toml(app: &App) -> Result<Option<Manifest>> {
        if app.includes_file("Cargo.toml") {
            let cargo_toml: Manifest = app.read_toml("Cargo.toml").context("Reading Cargo.toml")?;

            return Ok(Some(cargo_toml));
        }

        Ok(None)
    }

    fn get_rust_toolchain_file(app: &App) -> Option<String> {
        if app.includes_file("rust-toolchain") {
            Some("rust-toolchain".to_string())
        } else if app.includes_file("rust-toolchain.toml") {
            Some("rust-toolchain.toml".to_string())
        } else {
            None
        }
    }

    // Get the rust package version by parsing the `rust-version` field in `Cargo.toml`
    fn get_rust_pkg(app: &App, env: &Environment) -> Result<Pkg> {
        if let Some(version) = env.get_config_variable("RUST_VERSION") {
            return Ok(Pkg::new(&format!("rust-bin.stable.\"{version}\".default")));
        }

        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app) {
            return Ok(Pkg::new(&format!(
                "(rust-bin.fromRustupToolchainFile ../{toolchain_file})"
            )));
        }

        let pkg = match RustProvider::parse_cargo_toml(app)? {
            Some(toml_file) => toml_file.package.map_or_else(
                || Pkg::new(DEFAULT_RUST_PACKAGE),
                |package| {
                    package.rust_version.map_or_else(
                        || Pkg::new(DEFAULT_RUST_PACKAGE),
                        |version| {
                            Pkg::new(
                                format!("rust-bin.stable.\"{}\".default", version.get().unwrap())
                                    .as_str(),
                            )
                        },
                    )
                },
            ),
            None => Pkg::new(DEFAULT_RUST_PACKAGE),
        };

        Ok(pkg)
    }

    fn should_make_wasm32_wasi(app: &App, _env: &Environment) -> bool {
        let re_target = Regex::new(r#"target\s*=\s*"wasm32-wasi""#).expect("BUG: Broken regex");

        matches!(app.find_match(&re_target, ".cargo/config.toml"), Ok(true))
    }

    fn should_use_musl(app: &App, env: &Environment) -> Result<bool> {
        if RustProvider::should_make_wasm32_wasi(app, env) {
            return Ok(false);
        }

        if env.is_config_variable_truthy("NO_MUSL") {
            return Ok(false);
        }

        if RustProvider::get_rust_toolchain_file(app).is_some() {
            return Ok(false);
        }

        // Do not build for the musl target if using openssl
        if RustProvider::uses_openssl(app)? {
            return Ok(false);
        }

        Ok(true)
    }

    fn uses_openssl(app: &App) -> Result<bool> {
        // Check Cargo.toml
        if let Some(toml_file) = RustProvider::parse_cargo_toml(app)? {
            if toml_file.dependencies.contains_key("openssl")
                || toml_file.dev_dependencies.contains_key("openssl")
                || toml_file.build_dependencies.contains_key("openssl")
            {
                return Ok(true);
            }
        }

        // Check Cargo.lock
        if app.includes_file("Cargo.lock") && app.read_file("Cargo.lock")?.contains("openssl") {
            return Ok(true);
        }

        Ok(false)
    }

    fn resolve_cargo_workspace(app: &App, env: &Environment) -> Result<Option<String>> {
        if let Some(name) = env.get_config_variable("CARGO_WORKSPACE") {
            return Ok(Some(name));
        }

        if let Some(workspace) =
            RustProvider::parse_cargo_toml(app)?.and_then(|manifest| manifest.workspace)
        {
            if let Some(binary) = RustProvider::find_binary_in_workspace(app, &workspace)? {
                return Ok(Some(binary));
            }
        }

        Ok(None)
    }

    fn find_binary_in_workspace(app: &App, workspace: &Workspace) -> Result<Option<String>> {
        let find_binary = |member: &str| -> Result<Option<String>> {
            let mut manifest = app.read_toml::<Manifest>(&format!("{member}/Cargo.toml"))?;

            manifest.complete_from_path(&app.source.join(format!("{member}/Cargo.toml")))?;

            if let Some(package) = manifest.package {
                if !manifest.bin.is_empty() || manifest.lib.is_none() {
                    return Ok(Some(package.name));
                }
            }

            Ok(None)
        };

        for default_member in workspace
            .default_members
            .iter()
            .filter(|default_member| !workspace.exclude.contains(default_member))
        {
            // a member can have globs
            if default_member.contains('*') || default_member.contains('?') {
                for member in app.find_directories(default_member)? {
                    if let Some(bin) = find_binary(&member.to_string_lossy())? {
                        return Ok(Some(bin));
                    }
                }
            } else if let Some(bin) = find_binary(default_member)? {
                return Ok(Some(bin));
            }
        }

        for member in workspace
            .members
            .iter()
            .filter(|member| !workspace.exclude.contains(member))
        {
            // a member can have globs
            if member.contains('*') || member.contains('?') {
                for member in app.find_directories(member)? {
                    if let Some(bin) = find_binary(&member.to_string_lossy())? {
                        return Ok(Some(bin));
                    }
                }
            } else if let Some(bin) = find_binary(member)? {
                return Ok(Some(bin));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_no_version() -> Result<()> {
        assert_eq!(
            RustProvider::get_rust_pkg(
                &App::new("./examples/rust-rocket")?,
                &Environment::default()
            )?,
            Pkg::new(DEFAULT_RUST_PACKAGE)
        );

        Ok(())
    }

    #[test]
    fn test_custom_version() -> Result<()> {
        assert_eq!(
            RustProvider::get_rust_pkg(
                &App::new("./examples/rust-custom-version")?,
                &Environment::default()
            )?,
            Pkg::new("rust-bin.stable.\"1.56.0\".default")
        );

        Ok(())
    }

    #[test]
    fn test_toolchain_file() -> Result<()> {
        assert_eq!(
            RustProvider::get_rust_pkg(
                &App::new("./examples/rust-custom-toolchain")?,
                &Environment::default()
            )?,
            Pkg::new("(rust-bin.fromRustupToolchainFile ../rust-toolchain.toml)")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_environment_variable() -> Result<()> {
        assert_eq!(
            RustProvider::get_rust_pkg(
                &App::new("./examples/rust-custom-toolchain")?,
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_RUST_VERSION".to_string(),
                    "1.54.0".to_string()
                )]))
            )?,
            Pkg::new("rust-bin.stable.\"1.54.0\".default")
        );

        Ok(())
    }

    #[test]
    fn test_uses_openssl() -> Result<()> {
        assert!(!RustProvider::uses_openssl(&App::new(
            "./examples/rust-custom-version"
        )?)?,);
        assert!(RustProvider::uses_openssl(&App::new(
            "./examples/rust-openssl"
        )?)?,);

        Ok(())
    }
}
