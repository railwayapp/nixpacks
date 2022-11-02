use anyhow::{bail, Result};
use clap::{arg, Arg, Command};
use nixpacks::{
    create_docker_image, generate_build_plan,
    nixpacks::{
        builder::docker::DockerBuilderOptions,
        nix::pkg::Pkg,
        plan::{
            generator::GeneratePlanOptions,
            phase::{Phase, StartPhase},
            BuildPlan,
        },
    },
};
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    string::ToString,
};

enum PlanFormat {
    Json,
    Toml,
}

impl PlanFormat {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(PlanFormat::Json),
            "toml" => Ok(PlanFormat::Toml),
            _ => bail!("Invalid plan format"),
        }
    }
}

fn main() -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = Command::new("nixpacks")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(VERSION)
        .subcommand(
            Command::new("plan")
                .about("Generate a build plan for an app")
                .arg(arg!([PATH] "App source"))
                .arg(
                    Arg::new("format")
                        .short('f')
                        .takes_value(true)
                        .help("json|toml. Specify the output format of the plan"),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Create a docker image for an app")
                .arg(arg!([PATH] "App source"))
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Name for the built image")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .help("Save output directory instead of building it with Docker")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("dockerfile")
                        .long("dockerfile")
                        .help("Print the generated Dockerfile to stdout")
                        .hide(true),
                )
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .short('t')
                        .help("Additional tags to add to the output image")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("label")
                        .long("label")
                        .short('l')
                        .help("Additional labels to add to the output image")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("platform")
                        .long("platform")
                        .help("Set target platform for your output image")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("cache-key")
                        .long("cache-key")
                        .help(
                            "Unique identifier to key cache by. Defaults to the current directory",
                        )
                        .takes_value(true),
                )
                .arg(
                    Arg::new("current-dir")
                        .long("current-dir")
                        .help("Output Nixpacks related files to the current directory ")
                        .takes_value(false),
                )
                .arg(
                    Arg::new("no-cache")
                        .long("no-cache")
                        .help("Disable building with the cache"),
                )
                .arg(
                    Arg::new("incremental-cache-image")
                        .long("incremental-cache-image")
                        .help("Image to hold the cached directories between builds.")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .help("Display more info during build."),
                )
                .arg(
                    Arg::new("inline-cache")
                        .long("inline-cache")
                        .help("Enable writing cache metadata into the output image"),
                )
                .arg(
                    Arg::new("cache-from")
                        .long("cache-from")
                        .help("Image to consider as cache sources")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("no-error-without-start")
                        .long("no-error-without-start")
                        .help("Do not error when no start command can be found"),
                ),
        )
        .arg(
            Arg::new("json-plan")
                .long("json-plan")
                .help("Specify an entire build plan in json format that should be used to configure the build")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("install_cmd")
                .long("install-cmd")
                .short('i')
                .help("Specify the install command to use")
                .takes_value(true)
                .global(true),
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
            Arg::new("apt")
                .long("apt")
                .help("Provide additional apt packages to install in the environment")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .arg(
            Arg::new("libs")
                .long("libs")
                .help("Provide additional nix libraries to install in the environment")
                .takes_value(true)
                .multiple_values(true)
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
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to config file")
                .takes_value(true)
                .global(true),
        )
        .get_matches();

    let install_cmd = matches.value_of("install_cmd").map(|s| vec![s.to_string()]);
    let build_cmd = matches.value_of("build_cmd").map(|s| vec![s.to_string()]);
    let start_cmd = matches.value_of("start_cmd").map(ToString::to_string);
    let pkgs = match matches.values_of("pkgs") {
        Some(values) => values.map(Pkg::new).collect::<Vec<_>>(),
        None => Vec::new(),
    };
    let libs = match matches.values_of("libs") {
        Some(values) => values.map(String::from).collect::<Vec<String>>(),
        None => Vec::new(),
    };
    let apt_pkgs = match matches.values_of("apt") {
        Some(values) => values.map(String::from).collect::<Vec<String>>(),
        None => Vec::new(),
    };

    let envs: Vec<_> = match matches.values_of("env") {
        Some(envs) => envs.collect(),
        None => Vec::new(),
    };

    // CLI build plan
    let mut cli_plan = BuildPlan::default();
    if !pkgs.is_empty() || !libs.is_empty() || !apt_pkgs.is_empty() {
        let mut setup = Phase::setup(Some(vec![pkgs, vec![Pkg::new("...")]].concat()));
        setup.apt_pkgs = Some(vec![apt_pkgs, vec!["...".to_string()]].concat());
        setup.nix_libs = Some(vec![libs, vec!["...".to_string()]].concat());
        cli_plan.add_phase(setup);
    }
    if let Some(install_cmds) = install_cmd {
        let mut install = Phase::install(None);
        install.cmds = Some(install_cmds);
        cli_plan.add_phase(install);
    }
    if let Some(build_cmds) = build_cmd {
        let mut build = Phase::build(None);
        build.cmds = Some(build_cmds);
        cli_plan.add_phase(build);
    }
    if let Some(start_cmd) = start_cmd {
        let start = StartPhase::new(start_cmd);
        cli_plan.set_start_phase(start);
    }

    let json_plan = match matches.value_of("json-plan") {
        Some(json) => Some(BuildPlan::from_json(json)?),
        None => None,
    };

    // Merge the CLI build plan with the json build plan
    let cli_plan = if let Some(json_plan) = json_plan {
        BuildPlan::merge_plans(&[json_plan, cli_plan])
    } else {
        cli_plan
    };

    let config_file = matches.value_of("config").map(ToString::to_string);
    let options = GeneratePlanOptions {
        plan: Some(cli_plan),
        config_file,
    };

    match &matches.subcommand() {
        Some(("plan", matches)) => {
            let path = matches.value_of("PATH").unwrap_or(".");
            let format = PlanFormat::from_str(matches.value_of("format").unwrap_or("json"))?;

            let plan = generate_build_plan(path, envs, &options)?;

            let plan_s = match format {
                PlanFormat::Json => plan.to_json()?,
                PlanFormat::Toml => plan.to_toml()?,
            };

            println!("{}", plan_s);
        }
        Some(("build", matches)) => {
            let path = matches.value_of("PATH").unwrap_or(".");
            let name = matches.value_of("name").map(ToString::to_string);
            let out_dir = matches.value_of("out").map(ToString::to_string);
            let current_dir = matches.is_present("current-dir");
            let mut cache_key = matches.value_of("cache-key").map(ToString::to_string);
            let no_cache = matches.is_present("no-cache");
            let inline_cache = matches.is_present("inline-cache");
            let verbose = matches.is_present("verbose") || envs.contains(&"NIXPACKS_VERBOSE=1");

            let cache_from = if !no_cache {
                matches.value_of("cache-from").map(ToString::to_string)
            } else {
                None
            };

            let incremental_cache_image = matches
                .value_of("incremental-cache-image")
                .map(ToString::to_string);

            // Default to absolute `path` of the source that is being built as the cache-key if not disabled
            if !no_cache && cache_key.is_none() {
                cache_key = get_default_cache_key(path)?;
            }

            let print_dockerfile = matches.is_present("dockerfile");

            let tags = matches
                .values_of("tag")
                .map(|values| values.map(ToString::to_string).collect::<Vec<_>>())
                .unwrap_or_default();

            let labels = matches
                .values_of("label")
                .map(|values| values.map(ToString::to_string).collect::<Vec<_>>())
                .unwrap_or_default();
            let platform = matches
                .values_of("platform")
                .map(|values| values.map(ToString::to_string).collect::<Vec<_>>())
                .unwrap_or_default();

            let no_error_without_start = matches.is_present("no-error-without-start");

            let build_options = &DockerBuilderOptions {
                name,
                tags,
                labels,
                out_dir,
                quiet: false,
                cache_key,
                no_cache,
                platform,
                print_dockerfile,
                current_dir,
                inline_cache,
                cache_from,
                no_error_without_start,
                incremental_cache_image,
                verbose,
            };

            create_docker_image(path, envs, &options, build_options)?;
        }
        _ => eprintln!("Invalid command"),
    }

    Ok(())
}

fn get_default_cache_key(path: &str) -> Result<Option<String>> {
    let current_dir = env::current_dir()?;
    let source = current_dir.join(path).canonicalize();
    if let Ok(source) = source {
        let source_str = source.to_string_lossy().to_string();
        let mut hasher = DefaultHasher::new();
        source_str.hash(&mut hasher);

        let encoded_source = base64::encode(hasher.finish().to_be_bytes())
            .replace(|c: char| !c.is_alphanumeric(), "");

        Ok(Some(encoded_source))
    } else {
        Ok(None)
    }
}
