use super::ImageBuilder;

#[derive(Clone, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct DockerBuilderOptions {
    pub name: Option<String>,
    pub out_dir: Option<String>,
    pub print_dockerfile: bool,
    pub tags: Vec<String>,
    pub labels: Vec<String>,
    pub quiet: bool,
    pub cache_key: Option<String>,
    pub no_cache: bool,
    pub inline_cache: bool,
    pub cache_from: Option<String>,
    pub platform: Vec<String>,
    pub current_dir: bool,
    pub no_error_without_start: bool,
    pub incremental_cache_image: Option<String>,
    pub verbose: bool,
}

mod cache;
pub mod docker_image_builder;
mod dockerfile_generation;
pub mod file_server;
pub mod incremental_cache;
pub mod utils;
