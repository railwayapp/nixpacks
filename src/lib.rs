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
    plan::{generator::NixpacksBuildPlanGenerator, BuildPlan, PlanGenerator},
};
use anyhow::{bail, Result, Error};
use nixpacks::plan::generator::GeneratePlanOptions;
use providers::{
    clojure::ClojureProvider, crystal::CrystalProvider, csharp::CSharpProvider, dart::DartProvider,
    deno::DenoProvider, elixir::ElixirProvider, fsharp::FSharpProvider, go::GolangProvider,
    haskell::HaskellStackProvider, java::JavaProvider, node::NodeProvider, php::PhpProvider,
    python::PythonProvider, ruby::RubyProvider, rust::RustProvider, staticfile::StaticfileProvider,
    swift::SwiftProvider, zig::ZigProvider, Provider,
};
use futures_util::stream::StreamExt;
use actix_multipart::Multipart;
use actix_web::{
    web, get/*macro*/, post/*macro*/,
    App, HttpResponse, HttpServer, Responder, Result as ActixResult
};
use actix_web::error::ParseError;
use actix_web::http::ContentEncoding;
use actix_web::http::header::ContentDisposition;
use actix_web::middleware::{Compress, Logger};
use analysis_engine::enums::Log;
use analysis_engine::taxonomy;
use std::env;
use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::PathBuf;


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

    if let Some(ref phase) = plan.start_phase {
        if phase.cmd.is_none() && !build_options.no_error_without_start {
            bail!("No start command could be found")
        }
    }

    let logger = Logger::new();
    let builder = DockerImageBuilder::new(logger, build_options.clone());
    if build_options.buildtime_cache {

    }

    builder.create_image(app.source.to_str().unwrap(), &plan, &environment)?;

    Ok(())
}


async fn save_file(mut payload: Multipart, save_to: String) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let filepath = format!("{save_to}/{filename}");

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await??;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }

       

    }

    Ok(HttpResponse::Ok().into())
}

fn extract_files(filepath: String){
    let file = File::open(filepath)?;
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
    .entries()?
    .filter_map(|e| e.ok())
    .map(|mut entry| -> Result<PathBuf> {
        let path = entry.path()?.strip_prefix(prefix)?.to_owned();
        entry.unpack(&path)?;
        Ok(path)
    })
    .filter_map(|e| e.ok())
    .for_each(|x| println!("> {}", x.display()));
}

async fn start_files_receiving_server(save_to: String) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::fs::create_dir_all(save_to)?;

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/")
                .route(web::post().to(|m| save_file(m, save_to))),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;

    Ok(())
}


async fn upload(mut payload: Multipart, save_to: PathBuf) -> Result<HttpResponse, Error> {
    let mut output: Vec<String> = vec![];
    while let Some(item) = payload.next().await {
        let mut byte_stream_field = item?;
        let filename = byte_stream_field
            .content_disposition()
            .get_filename()
            .ok_or_else(|| ParseError::Incomplete)?;

        let filepath = save_to.join(sanitize_filename::sanitize(&filename));       
        // File::create is a blocking operation, so use a thread pool
        let in_path = PathBuf::from(&filepath);
        let mut f: File = web::block(|| File::create(in_path)).await??;
        while let Some(chunk) = byte_stream_field.next().await {
            let data = chunk?;
            // Writing a file is also a blocking operation, so use a thread pool
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }

        web::block(move || f.flush()).await??;

        // Process the provided zip file:
        
    }

    // Send the output zip file back to the client
   
}