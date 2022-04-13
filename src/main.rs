use ::nixpacks::{build, gen_plan};
use anyhow::Result;
use clap::{arg, Arg, Command};

fn main() -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = Command::new("nixpacks")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(VERSION)
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
                )
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .help("Save output directory instead of building it with Docker")
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
        .arg(
            Arg::new("env")
                .long("env")
                .help("Provide environment variables to your build")
                .takes_value(true)
                .multiple_values(true)
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

    let envs: Vec<_> = match matches.values_of("env") {
        Some(envs) => envs.collect(),
        None => Vec::new(),
    };

    match &matches.subcommand() {
        Some(("plan", matches)) => {
            let path = matches.value_of("PATH").expect("required");

            let plan = gen_plan(path, pkgs, build_cmd, start_cmd, envs, pin_pkgs)?;
            let json = serde_json::to_string_pretty(&plan)?;
            println!("{}", json);
        }
        Some(("build", matches)) => {
            let path = matches.value_of("PATH").expect("required");
            let name = matches.value_of("name").map(|n| n.to_string());
            let plan_path = matches.value_of("plan").map(|n| n.to_string());
            let output_dir = matches.value_of("out").map(|n| n.to_string());

            build(
                path, name, pkgs, build_cmd, start_cmd, pin_pkgs, envs, plan_path, output_dir,
            )?;
        }
        _ => eprintln!("Invalid command"),
    }

    Ok(())
}
