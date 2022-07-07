use std::env::consts::ARCH;

use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

static RUST_OVERLAY: &str = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
static DEFAULT_RUST_PACKAGE: &str = "rust-bin.stable.latest.default";

const CARGO_GIT_CACHE_DIR: &'static &str = &"/root/.cargo/git";
const CARGO_REGISTRY_CACHE_DIR: &'static &str = &"/root/.cargo/registry";
const CARGO_TARGET_CACHE_DIR: &'static &str = &"target";

#[derive(Serialize, Deserialize, Debug)]
pub struct CargoTomlPackage {
    pub name: String,
    pub version: String,
    #[serde(rename = "rust-version")]
    pub rust_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CargoToml {
    pub package: CargoTomlPackage,
}

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
            rust_pkg = rust_pkg.set_override("targets", format!("[\"{target}\"]").as_str());
        }

        let mut setup_phase =
            SetupPhase::new(vec![Pkg::new("gcc"), rust_pkg.from_overlay(RUST_OVERLAY)]);

        // Include the rust toolchain file so we can install that rust version with Nix
        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app)? {
            setup_phase.add_file_dependency(toolchain_file);
        }

        if !env.is_config_variable_truthy("NO_MUSL") {
            setup_phase.add_apt_pkgs(vec!["musl-tools".to_string()]);
        }

        Ok(Some(setup_phase))
    }

    fn build(&self, app: &App, env: &Environment) -> Result<Option<BuildPhase>> {
        let mut build_phase = match RustProvider::get_target(app, env)? {
            Some(target) => {
                let mut build_phase =
                    BuildPhase::new(format!("cargo build --release --target {target}"));

                if let Some(name) = RustProvider::get_app_name(app)? {
                    // Copy the binary out of the target directory
                    build_phase.add_cmd(format!("cp target/{target}/release/{name} {name}"));
                }

                build_phase
            }
            None => {
                let mut build_phase = BuildPhase::new("cargo build --release".to_string());

                if let Some(name) = RustProvider::get_app_name(app)? {
                    // Copy the binary out of the target directory
                    build_phase.add_cmd(format!("cp target/release/{name} {name}"));
                }

                build_phase
            }
        };

        build_phase.add_cache_directory(CARGO_GIT_CACHE_DIR.to_string());
        build_phase.add_cache_directory(CARGO_REGISTRY_CACHE_DIR.to_string());

        if RustProvider::get_app_name(app)?.is_some() {
            // Cache target directory
            build_phase.add_cache_directory(CARGO_TARGET_CACHE_DIR.to_string());
        }

        Ok(Some(build_phase))
    }

    fn start(&self, app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        let name = RustProvider::get_app_name(app)?;

        if let Some(name) = name {
            let start_phase = match RustProvider::get_target(app, env)? {
                Some(_) => {
                    let binary_file = format!("./{name}");
                    let mut start_phase = StartPhase::new(format!("./{name}"));

                    start_phase.run_in_slim_image();
                    start_phase.add_file_dependency(binary_file);

                    start_phase
                }
                None => StartPhase::new(format!("./{name}")),
            };

            Ok(Some(start_phase))
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
            let name = toml_file.package.name;
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }

    fn get_target(_app: &App, env: &Environment) -> Result<Option<String>> {
        // All the user to use the default target instead of compiling with musl
        if !env.is_config_variable_truthy("NO_MUSL") {
            Ok(Some(format!("{}-unknown-linux-musl", ARCH)))
        } else {
            Ok(None)
        }
    }

    fn parse_cargo_toml(app: &App) -> Result<Option<CargoToml>> {
        if app.includes_file("Cargo.toml") {
            let cargo_toml: CargoToml =
                app.read_toml("Cargo.toml").context("Reading Cargo.toml")?;
            return Ok(Some(cargo_toml));
        }

        Ok(None)
    }

    fn get_rust_toolchain_file(app: &App) -> Result<Option<String>> {
        if app.includes_file("rust-toolchain") {
            Ok(Some("rust-toolchain".to_string()))
        } else if app.includes_file("rust-toolchain.toml") {
            Ok(Some("rust-toolchain.toml".to_string()))
        } else {
            Ok(None)
        }
    }

    // Get the rust package version by parsing the `rust-version` field in `Cargo.toml`
    fn get_rust_pkg(app: &App, env: &Environment) -> Result<Pkg> {
        if let Some(version) = env.get_config_variable("RUST_VERSION") {
            return Ok(Pkg::new(
                format!("rust-bin.stable.\"{}\".default", version).as_str(),
            ));
        }

        if let Some(toolchain_file) = RustProvider::get_rust_toolchain_file(app)? {
            return Ok(Pkg::new(
                format!("(rust-bin.fromRustupToolchainFile ./{})", toolchain_file).as_str(),
            ));
        }

        let pkg = match RustProvider::parse_cargo_toml(app)? {
            Some(toml_file) => {
                let version = toml_file.package.rust_version;

                version
                    .map(|version| {
                        Pkg::new(format!("rust-bin.stable.\"{}\".default", version).as_str())
                    })
                    .unwrap_or_else(|| Pkg::new(DEFAULT_RUST_PACKAGE))
            }
            None => Pkg::new(DEFAULT_RUST_PACKAGE),
        };

        Ok(pkg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

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
                &Environment::new(HashMap::from([(
                    "NIXPACKS_RUST_VERSION".to_string(),
                    "1.54.0".to_string()
                )]))
            )?,
            Pkg::new("rust-bin.stable.\"1.54.0\".default")
        );

        Ok(())
    }
}
