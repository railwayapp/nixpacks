use crate::nixpacks::{app::App, environment::Environment, plan::BuildPlan};
use anyhow::Result;

pub mod clojure;
pub mod crystal;
pub mod csharp;
pub mod dart;
pub mod deno;
pub mod fsharp;
pub mod go;
pub mod haskell;
pub mod java;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod staticfile;
pub mod swift;
pub mod zig;

pub trait Provider {
    fn name(&self) -> &str;
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool>;
    fn get_build_plan(&self, _app: &App, _environment: &Environment) -> Result<Option<BuildPlan>>;
}
