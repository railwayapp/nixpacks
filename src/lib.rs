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
    crystal::CrystalProvider, csharp::CSharpProvider, dart::DartProvider, deno::DenoProvider,
    fsharp::FSharpProvider, go::GolangProvider, haskell::HaskellStackProvider, java::JavaProvider,
    node::NodeProvider, python::PythonProvider, rust::RustProvider, staticfile::StaticfileProvider,
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
        &CSharpProvider {},
        &FSharpProvider {},
        &CrystalProvider {},
        &StaticfileProvider {},
        &DartProvider {},
        &JavaProvider {},
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
    builder.create_image(app.source.to_str().unwrap(), &plan)?;

    Ok(())
}
