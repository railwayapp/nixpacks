use std::fs;

use anyhow::{Context, Result};
use bb::{app::App, logger::Logger, plan::BuildPlan, AppBuilder, AppBuilderOptions};
use clap::{arg, Arg, Command};
use providers::{go::GolangProvider, npm::NpmProvider, yarn::YarnProvider, Pkg, Provider};
mod bb;
mod providers;

fn get_providers() -> Vec<&'static dyn Provider> {
    vec![&YarnProvider {}, &NpmProvider {}, &GolangProvider {}]
}

fn main() -> Result<()> {
    let matches = Command::new("bb")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("plan")
                .about("Generate a build plan for an app")
                .arg(arg!(<PATH> "App source")),
        )
        .subcommand(
            Command::new("build")
                .about("Create a docker image for an app")
                .arg(arg!(<PATH> "App source"))
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Name for the built image")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("plan")
                        .long("plan")
                        .help("Existing build plan file to use")
                        .takes_value(true),
                ),
        )
        .arg(
            Arg::new("build_cmd")
                .long("build-cmd")
                .short('b')
                .help("Specify the build command to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("start_cmd")
                .long("start-cmd")
                .short('s')
                .help("Specify the start command to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("pkgs")
                .long("pkgs")
                .short('p')
                .help("Provide additional nix packages to install in the environment")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .arg(
            Arg::new("pin")
                .long("pin")
                .help("Pin the nixpkgs")
                .takes_value(false)
                .global(true),
        )
        .get_matches();

    let build_cmd = matches.value_of("build_cmd").map(|s| s.to_string());
    let start_cmd = matches.value_of("start_cmd").map(|s| s.to_string());
    let pkgs: Vec<_> = match matches.values_of("pkgs") {
        Some(values) => values.collect(),
        None => Vec::new(),
    };
    let pin_pkgs = matches.is_present("pin");

    let options = AppBuilderOptions {
        custom_pkgs: pkgs.iter().map(|p| Pkg::new(p)).collect(),
        custom_build_cmd: build_cmd,
        custom_start_cmd: start_cmd,
        pin_pkgs,
    };

    let logger = Logger::new();
    let providers = get_providers();

    match &matches.subcommand() {
        Some(("plan", matches)) => {
            let path = matches.value_of("PATH").expect("required");

            let app = App::new(path)?;
            let mut app_builder = AppBuilder::new(None, &app, &logger, &options)?;

            let plan = app_builder.plan(providers)?;
            let json = serde_json::to_string_pretty(&plan)?;
            println!("{}", json);
        }
        Some(("build", matches)) => {
            let path = matches.value_of("PATH").expect("required");
            let name = matches.value_of("name").map(|n| n.to_string());

            let app = App::new(path)?;
            let mut app_builder = AppBuilder::new(name, &app, &logger, &options)?;

            let plan_path = matches.value_of("plan");
            match plan_path {
                Some(plan_path) => {
                    let plan_json = fs::read_to_string(plan_path).context("Reading build plan")?;
                    let plan: BuildPlan =
                        serde_json::from_str(&plan_json).context("Deserializing build plan")?;
                    app_builder.build_from_plan(&plan)?;
                }
                None => {
                    app_builder.build(providers)?;
                }
            }
        }
        _ => eprintln!("Invalid command"),
    }

    Ok(())
}
