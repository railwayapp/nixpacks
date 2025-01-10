use anyhow::Result;
use clap::{arg, Parser, Subcommand, ValueEnum};
use nixpacks::{
    create_docker_image, generate_build_plan, get_plan_providers,
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
    ops::Deref,
    string::ToString,
};

/// The build plan config file format to use.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PlanFormat {
    Json,
    Toml,
}

/// Arguments passed to `nixpacks`.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Specify an entire build plan in json format that should be used to configure the build
    #[arg(long, global = true)]
    json_plan: Option<String>,

    /// Specify the install command to use
    #[arg(long, short, global = true)]
    install_cmd: Option<String>,

    /// Specify the build command to use
    #[arg(long, short, global = true)]
    build_cmd: Option<String>,

    /// Specify the start command to use
    #[arg(long, short, global = true)]
    start_cmd: Option<String>,

    /// Provide additional nix packages to install in the environment
    #[arg(long, short, global = true)]
    pkgs: Vec<String>,

    /// Provide additional apt packages to install in the environment
    #[arg(long, short, global = true)]
    apt: Vec<String>,

    /// Provide additional nix libraries to install in the environment
    #[arg(long, global = true)]
    libs: Vec<String>,

    /// Provide environment variables to your build
    #[arg(long, short, global = true)]
    env: Vec<String>,

    /// Path to config file
    #[arg(long, short, global = true)]
    config: Option<String>,
}

/// The valid subcommands passed to `nixpacks`, and their arguments.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Generate a build plan for an app.
    /// Generated plan will be outputted to stdout, while warnings might be outputted to stderr.
    Plan {
        /// App source
        path: String,

        /// Specify the output format of the build plan.
        #[arg(short, long, value_enum, default_value = "json")]
        format: PlanFormat,
    },

    /// List all of the providers that will be used to build the app
    Detect {
        /// App source
        path: String,
    },

    /// Build an app
    Build {
        /// App source
        path: String,

        /// Name for the built image
        #[arg(short, long)]
        name: Option<String>,

        /// Save output directory instead of building it with Docker
        #[arg(short, long)]
        out: Option<String>,

        /// Print the generated Dockerfile to stdout
        #[arg(short, long, hide = true)]
        dockerfile: bool,

        /// Additional tags to add to the output image
        #[arg(short, long)]
        tag: Vec<String>,

        /// Additional labels to add to the output image
        #[arg(short, long)]
        label: Vec<String>,

        /// Set target platform for your output image
        #[arg(long)]
        platform: Vec<String>,

        /// Unique identifier to key cache by. Defaults to the current directory
        #[arg(long)]
        cache_key: Option<String>,

        /// Output Nixpacks related files to the current directory
        #[arg(long)]
        current_dir: bool,

        /// Disable building with the cache
        #[arg(long)]
        no_cache: bool,

        /// Image to hold the cached directories between builds.
        #[arg(long)]
        incremental_cache_image: Option<String>,

        /// Image to consider as cache sources
        #[arg(long)]
        cache_from: Option<String>,

        /// Specify host for Docker client
        #[arg(long)]
        docker_host: Option<String>,

        /// Adds hosts to the Docker build
        #[arg(long, global = true)]
        add_host: Vec<String>,

        /// Specify if Docker client should verify the TLS (Transport Layer Security) certificates
        #[arg(long)]
        docker_tls_verify: Option<String>,

        /// Specify output destination for Docker build.
        /// https://docs.docker.com/reference/cli/docker/buildx/build/#output
        #[arg(long)]
        docker_output: Option<String>,

        /// Specify the path to the Docker client certificates
        #[arg(long)]
        docker_cert_path: Option<String>,

        /// Enable writing cache metadata into the output image
        #[arg(long)]
        inline_cache: bool,

        /// Do not error when no start command can be found
        #[arg(long)]
        no_error_without_start: bool,

        /// Limit the CPU CFS (Completely Fair Scheduler) quota.
        /// Passed directly to the docker build command
        #[arg(long)]
        cpu_quota: Option<String>,

        /// Memory limit.
        /// Passed directly to the docker build command
        #[arg(long)]
        memory: Option<String>,

        /// Display more info during build
        #[arg(long, short)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let pkgs = args
        .pkgs
        .iter()
        .map(|p| p.deref())
        .map(Pkg::new)
        .collect::<Vec<_>>();

    // CLI build plan
    let mut cli_plan = BuildPlan::default();
    if !args.pkgs.is_empty() || !args.libs.is_empty() || !args.apt.is_empty() {
        let mut setup = Phase::setup(Some([pkgs, [Pkg::new("...")].to_vec()].to_vec().concat()));
        setup.apt_pkgs = Some([args.apt, ["...".to_string()].to_vec()].to_vec().concat());
        setup.nix_libs = Some([args.libs, ["...".to_string()].to_vec()].to_vec().concat());
        cli_plan.add_phase(setup);
    }
    if let Some(install_cmds) = args.install_cmd {
        let mut install = Phase::install(None);
        install.cmds = Some(vec![install_cmds]);
        cli_plan.add_phase(install);
    }
    if let Some(build_cmds) = args.build_cmd {
        let mut build = Phase::build(None);
        build.cmds = Some(vec![build_cmds]);
        cli_plan.add_phase(build);
    }
    if let Some(start_cmd) = args.start_cmd {
        let start = StartPhase::new(start_cmd);
        cli_plan.set_start_phase(start);
    }

    let json_plan = args.json_plan.map(BuildPlan::from_json).transpose()?;

    // Merge the CLI build plan with the json build plan
    let cli_plan = if let Some(json_plan) = json_plan {
        BuildPlan::merge_plans(&[json_plan, cli_plan])
    } else {
        cli_plan
    };

    let env: Vec<&str> = args.env.iter().map(|e| e.deref()).collect();
    let options = GeneratePlanOptions {
        plan: Some(cli_plan),
        config_file: args.config,
    };

    match args.command {
        // Produce a build plan for a project and print it to stdout.
        Commands::Plan { path, format } => {
            let plan = generate_build_plan(&path, env, &options)?;

            let plan_s = match format {
                PlanFormat::Json => plan.to_json()?,
                PlanFormat::Toml => plan.to_toml()?,
            };

            println!("{plan_s}");
        }
        // Detect which providers should be used to build a project and print them to stdout.
        Commands::Detect { path } => {
            let providers = get_plan_providers(&path, env, &options)?;
            println!("{}", providers.join(", "));
        }
        // Generate a Dockerfile and builds a container, using any specified build options.
        Commands::Build {
            path,
            name,
            out,
            dockerfile,
            tag,
            label,
            platform,
            cache_key,
            current_dir,
            no_cache,
            incremental_cache_image,
            cache_from,
            docker_host,
            docker_tls_verify,
            docker_output,
            add_host,
            docker_cert_path,
            inline_cache,
            no_error_without_start,
            cpu_quota,
            memory,
            verbose,
        } => {
            let verbose = verbose || args.env.contains(&"NIXPACKS_VERBOSE=1".to_string());

            // Default to absolute `path` of the source that is being built as the cache-key if not disabled
            let cache_key = if !no_cache && cache_key.is_none() {
                get_default_cache_key(&path)?
            } else {
                cache_key
            };

            let build_options = &DockerBuilderOptions {
                name,
                tags: tag,
                labels: label,
                out_dir: out,
                quiet: false,
                cache_key,
                no_cache,
                platform,
                print_dockerfile: dockerfile,
                current_dir,
                inline_cache,
                cache_from,
                docker_host,
                docker_tls_verify,
                docker_output,
                docker_cert_path,
                no_error_without_start,
                incremental_cache_image,
                cpu_quota,
                add_host,
                memory,
                verbose,
            };
            create_docker_image(&path, env, &options, build_options).await?;
        }
    }

    Ok(())
}

/// Creates a key for storing image layers in the Docker cache.
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
