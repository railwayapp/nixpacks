use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::{NixConfig, Pkg},
};
use anyhow::{Context, Result};

static RUST_OVERLAY: &str = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";

pub struct RustProvider {}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Cargo.toml"))
    }

    fn pkgs(&self, _app: &App, _env: &Environment) -> Result<NixConfig> {
        Ok(NixConfig::new(vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.gcc"),
            Pkg::new("rust-bin.stable.latest.default"),
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
        if app.includes_file("Cargo.toml") {
            // Parse name from Cargo.toml so we can run ./target/release/{name}
            let toml_file: toml::Value =
                app.read_toml("Cargo.toml").context("Reading Cargo.toml")?;
            let name = toml_file
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(|v| v.as_str());

            if let Some(name) = name {
                return Ok(Some(format!("./target/release/{}", name)));
            }
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
