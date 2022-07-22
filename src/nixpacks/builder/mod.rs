use super::{environment::Environment, plan::BuildPlan};
use anyhow::Result;

pub mod docker;

pub trait ImageBuilder {
    fn create_image(&self, app_source: &str, plan: &BuildPlan, env: &Environment) -> Result<()>;
}
