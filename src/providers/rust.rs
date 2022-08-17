use std::env::consts::ARCH;
use std::fmt::Write as _;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use cargo_toml::{Manifest, Workspace};

static RUST_OVERLAY: &str = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
static DEFAULT_RUST_PACKAGE: &str = "rust-bin.stable.latest.default";

const CARGO_GIT_CACHE_DIR: &'static &str = &"/root/.cargo/git";
const CARGO_REGISTRY_CACHE_DIR: &'static &str = &"/root/.cargo/registry";
const CARGO_TARGET_CACHE_DIR: &'static &str = &"target";

pub struct RustProvider {}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Cargo.toml"))
    }

    fn setup(&self, app: &App, env: &Environment) -> Result<Option<SetupPhase>> {
        let mut rust_pkg: Pkg = RustProvider::get_rust_pkg(app, env)?;

        if let Some(target) = RustProvider::get_target(app, env)? {
            rust_pkg = rust_pkg.set_override("targets", &format!("[\"{}\"]", target));
        }

        let mut setup_phase =
            SetupPhase::new(vec![Pkg::new("gcc"), rust_pkg.from_overlay(RUST_OVERLAY)]);

        // Include the rust toolchain file so we can install that rust version with Nix
        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app) {
            setup_phase.add_file_dependency(toolchain_file);
        }

        // Custom libs for openssl
        if RustProvider::uses_openssl(app)? {
            setup_phase.add_libraries(vec!["openssl".to_string(), "openssl.dev".to_string()]);
        }

        if RustProvider::should_use_musl(app, env)? {
            setup_phase.add_apt_pkgs(vec!["musl-tools".to_string()]);
        }

        setup_phase.add_apt_pkgs(vec!["binutils".to_string()]);

        Ok(Some(setup_phase))
    }

    fn build(&self, app: &App, env: &Environment) -> Result<Option<BuildPhase>> {
        let mut build_phase = RustProvider::get_build_phase(app, env)?;

        build_phase.add_cache_directory((*CARGO_GIT_CACHE_DIR).to_string());
        build_phase.add_cache_directory((*CARGO_REGISTRY_CACHE_DIR).to_string());

        if RustProvider::get_app_name(app)?.is_some() {
            // Cache target directory
            build_phase.add_cache_directory((*CARGO_TARGET_CACHE_DIR).to_string());
        }

        Ok(Some(build_phase))
    }

    fn start(&self, app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        if (RustProvider::get_target(app, env)?).is_some() {
            if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
                let mut start_phase = StartPhase::new(format!("./{}", workspace));

                start_phase.run_in_slim_image();
                start_phase.add_file_dependency(format!("./{}", workspace));

                Ok(Some(start_phase))
            } else if let Some(name) = RustProvider::get_app_name(app)? {
                let mut start_phase = StartPhase::new(format!("./{}", name));

                start_phase.run_in_slim_image();
                start_phase.add_file_dependency(format!("./{}", name));

                Ok(Some(start_phase))
            } else {
                Ok(None)
            }
        } else if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
            Ok(Some(StartPhase::new(format!("./{}", workspace))))
        } else if let Some(name) = RustProvider::get_app_name(app)? {
            Ok(Some(StartPhase::new(format!("./{}", name))))
        } else {
            Ok(None)
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        let mut variables = EnvironmentVariables::default();
        variables.insert("ROCKET_ADDRESS".to_string(), "0.0.0.0".to_string());
        Ok(Some(variables))
    }
}

impl RustProvider {
    fn get_app_name(app: &App) -> Result<Option<String>> {
        if let Some(toml_file) = RustProvider::parse_cargo_toml(app)? {
            if let Some(package) = toml_file.package {
                let name = package.name;
                return Ok(Some(name));
            }
        }

        Ok(None)
    }

    fn get_target(app: &App, env: &Environment) -> Result<Option<String>> {
        if RustProvider::should_use_musl(app, env)? {
            Ok(Some(format!("{}-unknown-linux-musl", ARCH)))
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
            return Ok(Pkg::new(&format!(
                "rust-bin.stable.\"{}\".default",
                version
            )));
        }

        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app) {
            return Ok(Pkg::new(&format!(
                "(rust-bin.fromRustupToolchainFile ./{})",
                toolchain_file
            )));
        }

        let pkg = match RustProvider::parse_cargo_toml(app)? {
            Some(toml_file) => toml_file.package.map_or_else(
                || Pkg::new(DEFAULT_RUST_PACKAGE),
                |package| {
                    package.rust_version.map_or_else(
                        || Pkg::new(DEFAULT_RUST_PACKAGE),
                        |version| {
                            Pkg::new(format!("rust-bin.stable.\"{}\".default", version).as_str())
                        },
                    )
                },
            ),
            None => Pkg::new(DEFAULT_RUST_PACKAGE),
        };

        Ok(pkg)
    }

    fn should_use_musl(app: &App, env: &Environment) -> Result<bool> {
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

        let manifest = RustProvider::parse_cargo_toml(app)?.context("Missing Cargo.toml")?;

        if let Some(workspace) = manifest.workspace {
            if let Some(binary) = RustProvider::find_binary_in_workspace(app, &workspace)? {
                return Ok(Some(binary));
            }
        }

        Ok(None)
    }

    fn find_binary_in_workspace(app: &App, workspace: &Workspace) -> Result<Option<String>> {
        let find_binary = |member: &str| -> Result<Option<String>> {
            let mut manifest = app.read_toml::<Manifest>(&format!("{}/Cargo.toml", member))?;

            manifest.complete_from_path(&app.source.join(format!("{}/Cargo.toml", member)))?;

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

    fn get_build_phase(app: &App, env: &Environment) -> Result<BuildPhase> {
        let mut build_cmd = "cargo build --release".to_string();

        if let Some(target) = RustProvider::get_target(app, env)? {
            if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
                write!(build_cmd, " --package {} --target {}", workspace, target)?;

                let mut build_phase = BuildPhase::new(build_cmd);

                build_phase.add_cmd(format!(
                    "cp target/{}/release/{name} {name}",
                    target,
                    name = workspace
                ));

                Ok(build_phase)
            } else {
                write!(build_cmd, " --target {}", target)?;

                let mut build_phase = BuildPhase::new(build_cmd);

                if let Some(name) = RustProvider::get_app_name(app)? {
                    build_phase.add_cmd(format!(
                        "cp target/{}/release/{name} {name}",
                        target,
                        name = name
                    ));
                }

                Ok(build_phase)
            }
        } else if let Some(workspace) = RustProvider::resolve_cargo_workspace(app, env)? {
            write!(build_cmd, " --package {}", workspace)?;

            let mut build_phase = BuildPhase::new(build_cmd);

            build_phase.add_cmd(format!("cp target/release/{name} {name}", name = workspace));

            Ok(build_phase)
        } else {
            let mut build_phase = BuildPhase::new(build_cmd);

            if let Some(name) = RustProvider::get_app_name(app)? {
                build_phase.add_cmd(format!("cp target/release/{name} {name}", name = name));
            }

            Ok(build_phase)
        }
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
            Pkg::new("(rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)")
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
