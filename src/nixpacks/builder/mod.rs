use super::{app::App, plan::BuildPlan};
use anyhow::Result;

pub mod docker;

pub trait Builder {
    fn create_image(&self, app: &App, plan: &BuildPlan) -> Result<()>;
}
