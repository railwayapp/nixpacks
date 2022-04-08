use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
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

    fn pkgs(&self, app: &App, _env: &Environment) -> Result<NixConfig> {
        let rust_pkg: Pkg = self.get_rust_pkg(app)?;

        Ok(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.gcc"),
            rust_pkg,
        ])
        .add_overlay(RUST_OVERLAY.to_string()))
    }

    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(Some("cargo build --release".to_string()))
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        if let Some(toml_file) = self.parse_cargo_toml(app)? {
            let name = toml_file.package.name;
            return Ok(Some(format!("./target/release/{}", name)));
        }

        Ok(None)
    }

    fn get_environment_variables(
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

    // Get the rust package version by parsing the `rust-version` field in `Cargo.toml`
    fn get_rust_pkg(&self, app: &App) -> Result<Pkg> {
        let pkg = match self.parse_cargo_toml(app)? {
            Some(toml_file) => {
                println!("{:?}", toml_file);
                let version = toml_file.package.rust_version;
                let pkg = version
                    .and_then(|version| {
                        Some(Pkg::new(
                            format!("rust-bin.stable.\"{}\".default", version).as_str(),
                        ))
                    })
                    .unwrap_or_else(|| Pkg::new(DEFAULT_RUST_PACKAGE));

                pkg
            }
            None => Pkg::new(DEFAULT_RUST_PACKAGE),
        };

        Ok(pkg)
    }
}
