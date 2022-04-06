use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod deno;
pub mod go;
pub mod npm;
pub mod rust;
pub mod yarn;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Pkg {
    pub name: String,
}

impl Pkg {
    pub fn new(name: &str) -> Pkg {
        Pkg {
            name: name.to_string(),
        }
    }
}

pub trait Provider {
    fn name(&self) -> &str;
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;
    fn pkgs(&self, app: &App, _env: &Environment) -> Vec<Pkg>;
    fn install_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }
    fn get_environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<EnvironmentVariables> {
        Ok(EnvironmentVariables::default())
    }
}
