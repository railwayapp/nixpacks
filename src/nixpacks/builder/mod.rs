use super::{environment::Environment, plan::BuildPlan};
use anyhow::Result;
use async_trait::async_trait;

pub mod docker;

/// Types that impl this trait can produce Docker images.
#[async_trait]
pub trait ImageBuilder {
    async fn create_image(
        &self,
        app_source: &str,
        plan: &BuildPlan,
        env: &Environment,
    ) -> Result<()>;
}
