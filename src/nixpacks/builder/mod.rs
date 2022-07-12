use super::{
    environment::Environment,
    plan::{new_build_plan::NewBuildPlan, BuildPlan},
};
use anyhow::Result;

pub mod docker;

pub trait Builder {
    fn create_image(&self, app_source: &str, plan: &NewBuildPlan, env: &Environment) -> Result<()>;
}
