#![warn(clippy::pedantic)]
#![allow(
    // Allowed as they are too pedantic.
    clippy::cast_possible_truncation,
    clippy::unreadable_literal,
    clippy::cast_possible_wrap,
    clippy::wildcard_imports,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::cast_lossless,
    clippy::unused_self,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    // TODO: Remove when everything is documented.
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
)]

use crate::nixpacks::{
    app::App,
    builder::{
        docker::{docker_image_builder::DockerImageBuilder, DockerBuilderOptions},
        ImageBuilder,
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
    clojure::ClojureProvider, cobol::CobolProvider, crystal::CrystalProvider,
    csharp::CSharpProvider, dart::DartProvider, deno::DenoProvider, elixir::ElixirProvider,
    fsharp::FSharpProvider, go::GolangProvider, haskell::HaskellStackProvider, java::JavaProvider,
    node::NodeProvider, php::PhpProvider, python::PythonProvider, ruby::RubyProvider,
    rust::RustProvider, staticfile::StaticfileProvider, swift::SwiftProvider, zig::ZigProvider,
    Provider,
};

mod chain;
#[macro_use]
pub mod nixpacks;
pub mod providers;

pub fn get_providers() -> &'static [&'static dyn Provider] {
    &[
        &CrystalProvider {},
        &CSharpProvider {},
        &DartProvider {},
        &ElixirProvider {},
        &DenoProvider {},
        &FSharpProvider {},
        &ClojureProvider {},
        &GolangProvider {},
        &HaskellStackProvider {},
        &JavaProvider {},
        &PhpProvider {},
        &RubyProvider {},
        &NodeProvider {},
        &PythonProvider {},
        &RustProvider {},
        &SwiftProvider {},
        &StaticfileProvider {},
        &ZigProvider {},
        &CobolProvider {},
    ]
}

pub fn generate_build_plan(
    path: &str,
    envs: Vec<&str>,
    options: &GeneratePlanOptions,
) -> Result<BuildPlan> {
    let app = App::new(path)?;
    let environment = Environment::from_envs(envs)?;

    let mut generator = NixpacksBuildPlanGenerator::new(get_providers(), options.clone());
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

    let mut generator = NixpacksBuildPlanGenerator::new(get_providers(), plan_options.clone());
    let plan = generator.generate_plan(&app, &environment)?;

    let logger = Logger::new();
    let builder = DockerImageBuilder::new(logger, build_options.clone());
    builder.create_image(app.source.to_str().unwrap(), &plan, &environment)?;

    Ok(())
}
