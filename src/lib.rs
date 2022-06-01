use crate::nixpacks::{
    app::App,
    builder::{
        docker::{DockerBuilder, DockerBuilderOptions},
        Builder,
    },
    environment::Environment,
    logger::Logger,
    nix::pkg::Pkg,
    plan::{
        generator::{GeneratePlanOptions, NixpacksBuildPlanGenerator},
        BuildPlan, PlanGenerator,
    },
};
use anyhow::Result;
use providers::{
    crystal::CrystalProvider, deno::DenoProvider, go::GolangProvider,
    haskell::HaskellStackProvider, node::NodeProvider, python::PythonProvider, rust::RustProvider,
    Provider,
};

mod chain;
pub mod nixpacks;
pub mod providers;

pub fn get_providers() -> Vec<&'static dyn Provider> {
    vec![
        &GolangProvider {},
        &DenoProvider {},
        &NodeProvider {},
        &RustProvider {},
        &PythonProvider {},
        &HaskellStackProvider {},
        &CrystalProvider {},
    ]
}

pub fn generate_build_plan(
    path: &str,
    envs: Vec<&str>,
    plan_options: &GeneratePlanOptions,
) -> Result<BuildPlan> {
    let app = App::new(path)?;
    let environment = Environment::from_envs(envs)?;

    let mut generator = NixpacksBuildPlanGenerator::new(get_providers(), plan_options.to_owned());
    let plan = generator.generate_plan(&app, &environment)?;

    Ok(plan)
}

pub fn create_docker_image(
    path: &str,
    envs: Vec<&str>,
    plan_options: &GeneratePlanOptions,
    build_options: &DockerBuilderOptions,
) -> Result<()> {
    let app = App::new(path)?;
    let environment = Environment::from_envs(envs)?;

    let mut generator = NixpacksBuildPlanGenerator::new(get_providers(), plan_options.to_owned());
    let plan = generator.generate_plan(&app, &environment)?;

    let logger = Logger::new();
    let builder = DockerBuilder::new(logger, build_options.to_owned());
    builder.create_image(&app, &plan)?;

    Ok(())
}

// pub fn gen_plan(
//     path: &str,
//     custom_pkgs: Vec<&str>,
//     custom_build_cmd: Option<String>,
//     custom_start_cmd: Option<String>,
//     envs: Vec<&str>,
//     pin_pkgs: bool,
// ) -> Result<BuildPlan> {
//     let logger = Logger::new();
//     let providers = get_providers();

//     let options = AppBuilderOptions {
//         custom_pkgs: custom_pkgs.iter().map(|p| Pkg::new(p)).collect(),
//         custom_build_cmd,
//         custom_start_cmd,
//         pin_pkgs,
//         out_dir: None,
//         plan_path: None,
//         tags: Vec::new(),
//         labels: Vec::new(),
//         quiet: false,
//     };

//     let app = App::new(path)?;
//     let environment = create_environment(envs)?;
//     let mut app_builder = AppBuilder::new(None, &app, &environment, &logger, &options)?;

//     let plan = app_builder.plan(providers)?;
//     Ok(plan)
// }

// #[allow(clippy::too_many_arguments)]
// pub fn build(
//     path: &str,
//     name: Option<String>,
//     custom_pkgs: Vec<&str>,
//     custom_build_cmd: Option<String>,
//     custom_start_cmd: Option<String>,
//     pin_pkgs: bool,
//     envs: Vec<&str>,
//     plan_path: Option<String>,
//     out_dir: Option<String>,
//     tags: Vec<&str>,
//     labels: Vec<&str>,
//     quiet: bool,
// ) -> Result<()> {
//     let logger = Logger::new();
//     let providers = get_providers();

//     let options = AppBuilderOptions {
//         custom_pkgs: custom_pkgs.iter().map(|p| Pkg::new(p)).collect(),
//         custom_build_cmd,
//         custom_start_cmd,
//         pin_pkgs,
//         out_dir,
//         plan_path,
//         tags: tags.iter().map(|s| s.to_string()).collect(),
//         labels: labels.iter().map(|s| s.to_string()).collect(),
//         quiet,
//     };

//     let app = App::new(path)?;
//     let environment = create_environment(envs)?;
//     let mut app_builder = AppBuilder::new(name, &app, &environment, &logger, &options)?;

//     app_builder.build(providers)?;

//     Ok(())
// }
