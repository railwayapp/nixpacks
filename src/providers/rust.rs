use super::Provider;
use crate::{nixpacks::app::App, providers::Pkg};
use anyhow::{Context, Result};
use toml::Value;

pub struct RustProvider {}

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, app: &App) -> Result<bool> {
        Ok(app.includes_file("Cargo.toml"))
    }

    fn pkgs(&self, _app: &App) -> Vec<Pkg> {
        vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.gcc"),
            Pkg::new("pkgs.rustc"),
            Pkg::new("pkgs.cargo"),
        ]
    }

    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(Some("cargo build --release".to_string()))
    }

    fn suggested_start_command(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("Cargo.toml") {
            // Parse name from Cargo.toml so we can run ./target/release/{name}
            let contents = app.read_file("Cargo.toml").context("Reading Cargo.toml")?;
            let toml_file = contents
                .parse::<Value>()
                .context("Parsing Cargo.toml file")?;

            let name = toml_file
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(|v| v.as_str());

            if let Some(name) = name {
                return Ok(Some(format!(
                    "ROCKET_ADDRESS=0.0.0.0 ./target/release/{}",
                    name
                )));
            }
        }

        Ok(None)
    }
}
