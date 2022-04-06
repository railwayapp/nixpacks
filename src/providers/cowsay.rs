use super::Provider;
use crate::{
    nixpacks::{app::App, environment::Environment},
    providers::Pkg,
};
use anyhow::Result;

pub struct CowsayProvider {}

impl Provider for CowsayProvider {
    fn name(&self) -> &str {
        "cowsay"
    }

    fn detect(&self, _app: &App, env: &Environment) -> Result<bool> {
        Ok(env.get_variable("SAY").is_some())
    }

    fn pkgs(&self, _app: &App, _env: &Environment) -> Vec<Pkg> {
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("pkgs.cowsay")]
    }

    fn suggested_start_command(&self, _app: &App, env: &Environment) -> Result<Option<String>> {
        let say = env
            .get_variable("SAY")
            .cloned()
            .unwrap_or_else(|| "".to_string());
        Ok(Some(format!("echo '{}' | cowsay", say)))
    }
}
