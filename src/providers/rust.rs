use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

static RUST_OVERLAY: &str = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
static DEFAULT_RUST_PACKAGE: &str = "rust-bin.stable.latest.default";

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

    fn setup(&self, app: &App, _env: &Environment) -> Result<SetupPhase> {
        let rust_pkg: Pkg = self.get_rust_pkg(app)?;

        let mut nix_config = NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.gcc"),
            rust_pkg,
        ]);
        nix_config.add_overlay(RUST_OVERLAY.to_string());

        let mut setup_phase = SetupPhase::new(nix_config);

        // Include the rust toolchain file so we can install that rust version with Nix
        if let Some(toolchain_file) = self.get_rust_toolchain_file(app)? {
            setup_phase.file_dependencies.push(toolchain_file);
        }

        Ok(setup_phase)
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<BuildPhase> {
        Ok(BuildPhase::new("cargo build --release".to_string()))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<StartPhase> {
        if let Some(toml_file) = self.parse_cargo_toml(app)? {
            let name = toml_file.package.name;
            Ok(StartPhase::new(format!("./target/release/{}", name)))
        } else {
            Ok(StartPhase::default())
        }
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        let mut variables = EnvironmentVariables::default();
        variables.insert("ROCKET_ADDRESS".to_string(), "0.0.0.0".to_string());
        Ok(variables)
    }
}

impl RustProvider {
    fn parse_cargo_toml(&self, app: &App) -> Result<Option<CargoToml>> {
        if app.includes_file("Cargo.toml") {
            let cargo_toml: CargoToml =
                app.read_toml("Cargo.toml").context("Reading Cargo.toml")?;
            return Ok(Some(cargo_toml));
        }

        Ok(None)
    }

    fn get_rust_toolchain_file(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("rust-toolchain") {
            Ok(Some("rust-toolchain".to_string()))
        } else if app.includes_file("rust-toolchain.toml") {
            Ok(Some("rust-toolchain.toml".to_string()))
        } else {
            Ok(None)
        }
    }

    // Get the rust package version by parsing the `rust-version` field in `Cargo.toml`
    fn get_rust_pkg(&self, app: &App) -> Result<Pkg> {
        if let Some(toolchain_file) = self.get_rust_toolchain_file(app)? {
            return Ok(Pkg::new(
                format!("(rust-bin.fromRustupToolchainFile ./{})", toolchain_file).as_str(),
            ));
        }

        let pkg = match self.parse_cargo_toml(app)? {
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
