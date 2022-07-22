use super::ImageBuilder;

#[derive(Clone, Default, Debug)]
pub struct DockerBuilderOptions {
    pub name: Option<String>,
    pub out_dir: Option<String>,
    pub print_dockerfile: bool,
    pub tags: Vec<String>,
    pub labels: Vec<String>,
    pub quiet: bool,
    pub cache_key: Option<String>,
    pub no_cache: bool,
    pub platform: Vec<String>,
}

mod cache;
pub mod docker_image_builder;
mod dockerfile_generation;
mod utils;
