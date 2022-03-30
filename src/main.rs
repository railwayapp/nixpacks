use anyhow::Result;
use bb::{app::App, logger::Logger, AppBuilder};
use clap::{arg, Arg, Command};
use providers::{go::GolangProvider, npm::NpmProvider, yarn::YarnProvider, Provider};
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
            Command::new("build")
                .about("Create a docker image based on app source")
                .arg(arg!(<PATH> "App source"))
                .arg(
                    Arg::new("build_cmd")
                        .long("build-cmd")
                        .short('b')
                        .help("Specify the build command to use")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("start_cmd")
                        .long("start-cmd")
                        .short('s')
                        .help("Specify the start command to use")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Name for the built image")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("pkgs")
                        .long("pkgs")
                        .short('p')
                        .help("Provide additional nix packages to install in the environment")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("nix")
                        .long("nix")
                        .help("Show the nix expression that would generated")
                        .takes_value(false),
                )
                .arg(
                    Arg::new("dockerfile")
                        .long("dockerfile")
                        .help("Show the Dockerfile that would be generated"),
                ),
        )
        .get_matches();

    match &matches.subcommand() {
        Some(("build", matches)) => {
            let path = matches.value_of("PATH").expect("required");

            let build_cmd = matches.value_of("build_cmd").map(|s| s.to_string());
            let start_cmd = matches.value_of("start_cmd").map(|s| s.to_string());
            let pkgs: Vec<_> = match matches.values_of("pkgs") {
                Some(values) => values.collect(),
                None => Vec::new(),
            };

            let name = matches.value_of("name").map(|n| n.to_string());

            let show_nix = matches.is_present("nix");
            let show_dockerfile = matches.is_present("dockerfile");

            let providers = get_providers();

            let app = App::new(path)?;
            let logger = Logger::new();

            let mut app_builder = AppBuilder::new(
                name,
                &app,
                &logger,
                build_cmd,
                start_cmd,
                pkgs.iter().map(|s| s.to_string()).collect(),
            )?;
            app_builder.detect(providers)?;

            if show_nix {
                let nix_expression = app_builder.gen_nix()?;
                println!("\n=== Nix Expression ===");
                println!("\n{}", nix_expression);
            }
            if show_dockerfile {
                let dockerfile = app_builder.gen_dockerfile()?;

                println!("\n=== Dockerfile ===");
                println!("\n{}", dockerfile);
            }

            if !show_nix && !show_dockerfile {
                app_builder.build()?;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
