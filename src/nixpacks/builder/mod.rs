use super::plan::BuildPlan;
use anyhow::Result;

pub mod docker;

pub trait Builder {
    fn create_image(&self, app_source: &str, plan: &BuildPlan) -> Result<()>;
}
