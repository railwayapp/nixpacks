use super::{app::App, plan::BuildPlan};
use anyhow::{bail, Context, Ok, Result};

pub mod docker;

pub trait Builder {
    fn create_image(&self, app: &App, plan: &BuildPlan) -> Result<()>;
}
