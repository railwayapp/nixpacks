use anyhow::Result;
use clap::{App, AppSettings};
use std::path::PathBuf;

use crate::bb::AppSource;

pub mod npm;
pub mod yarn;

pub trait Builder {
    fn name(&self) -> &str;
    fn detect(&self, app: &AppSource) -> Result<bool>;
    fn build_inputs(&self, app: &AppSource) -> String;
    fn install_cmd(&self, app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, app: &AppSource) -> Result<Option<String>> {
        Ok(None)
    }
}
