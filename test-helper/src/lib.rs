use dotenv_parser::parse_dotenv;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use rand::thread_rng;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use walkdir::{DirEntry, WalkDir};

const PLAN_TESTS_IGNORE: &[&str] = &[
    "rust-custom-version",
    "rust-rocket",
    "haskell-stack",
    "zig-gyro",
    "rust-ring",
    "rust-openssl",
    "rust-custom-toolchain",
    "rust-cargo-workspaces",
    "rust-cargo-workspaces-glob",
    "ruby-no-version",
];

fn get_examples(ignore: &[&str]) -> Vec<String> {
    let mut current_dir = std::env::current_dir().unwrap();
    let mut to_ignore = ignore.to_vec();

    to_ignore.push("examples");
    current_dir.push("examples");

    let walker = WalkDir::new(&current_dir).max_depth(1);

    walker
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .filter(|path| !to_ignore.contains(&path.as_str()))
        .collect()
}

fn gen_random_64_str() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

#[proc_macro]
pub fn generate_plan_tests(_tokens: TokenStream) -> TokenStream {
    let examples = get_examples(PLAN_TESTS_IGNORE);
    let mut tests = vec![quote! {
        macro_rules! assert_plan_snapshot {
            ($plan:expr) => {
                ::insta::assert_json_snapshot!($plan, {
                    ".nixpacksVersion" => "[version]",
                    ".buildImage" => "[build_image]",
                    ".phases.*.nixpacksArchive" => "[archive]",
                });
            }
        }

        fn simple_gen_plan(path: &str) -> ::nixpacks::nixpacks::plan::BuildPlan {
            if let Ok(raw_env) = ::std::fs::read_to_string(format!("{}/test.env", path)) {
                let env = ::dotenv_parser::parse_dotenv(&raw_env).unwrap();
                let opts = ::nixpacks::nixpacks::plan::config::GeneratePlanConfig {
                    pin_pkgs: env.get("TEST_PLAN_PIN_PKGS").is_some(),
                    custom_start_cmd: env.get("TEST_PLAN_CUSTOM_START_CMD").map(|cmd| cmd.to_string()),
                    custom_pkgs: env
                        .get("TEST_PLAN_CUSTOM_PKGS")
                        .map(|pkgs| pkgs.split(',')
                        .map(|pkg| ::nixpacks::nixpacks::nix::pkg::Pkg::new(pkg)).collect())
                        .unwrap_or_default(),
                    ..::nixpacks::nixpacks::plan::config::GeneratePlanConfig::default()
                };

                return ::nixpacks::generate_build_plan(
                    path,
                    env.get("TEST_PLAN_ENVS").map(|envs| envs.split(", ").collect()).unwrap_or_default(),
                    &opts
                ).unwrap();
            }

            ::nixpacks::generate_build_plan(
                path,
                Vec::new(),
                &::nixpacks::nixpacks::plan::config::GeneratePlanConfig::default()
            ).unwrap()
        }
    }];

    for example in examples {
        let test_name = format_ident!("{}", example.replace('-', "_"));
        let test = quote! {
            #[test]
            fn #test_name() {
                let plan = simple_gen_plan(&format!("./examples/{}", #example));
                assert_plan_snapshot!(plan);
            }
        };

        tests.push(test);
    }

    tests
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}

#[proc_macro]
pub fn generate_docker_tests(_tokens: TokenStream) -> TokenStream {
    let examples = get_examples(&[])
        .into_iter()
        .filter(|path| {
            if let Ok(env) = fs::read_to_string(format!("./examples/{}/test.env", path)) {
                env.contains("TEST_DOCKER")
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    let mut tests = vec![quote! {
        use ::anyhow::Context;
        use ::std::io::BufRead;
        use ::wait_timeout::ChildExt;

        fn get_container_ids_from_image(image: String) -> String {
            let output = ::std::process::Command::new("docker")
                .arg("ps")
                .arg("-a")
                .arg("-q")
                .arg("--filter")
                .arg(format!("ancestor={}", image))
                .output()
                .expect("failed to execute docker ps");

            String::from_utf8_lossy(&output.stdout).to_string()
        }

        fn stop_containers(container_id: &str) {
            ::std::process::Command::new("docker")
                .arg("stop")
                .arg(container_id)
                .spawn()
                .unwrap()
                .wait()
                .context("Stopping container")
                .unwrap();
        }

        fn remove_containers(container_id: &str) {
            ::std::process::Command::new("docker")
                .arg("rm")
                .arg(container_id)
                .spawn()
                .unwrap()
                .wait()
                .context("Removing container")
                .unwrap();
        }

        fn stop_and_remove_container_by_image(image: String) {
            let container_ids = get_container_ids_from_image(image);
            let container_id = container_ids.trim().split('\n').collect::<Vec<_>>()[0].to_string();

            stop_and_remove_container(container_id);
        }

        fn stop_and_remove_container(name: String) {
            stop_containers(&name);
            remove_containers(&name);
        }

        struct Config {
            environment_variables: ::nixpacks::nixpacks::environment::EnvironmentVariables,
            network: Option<String>,
        }

        /// Runs an image with Docker and returns the output
        /// The image is automatically stopped and removed after `TIMEOUT_SECONDS`
        fn run_image(name: String, cfg: Option<Config>) -> String {
            let mut cmd = ::std::process::Command::new("docker");
            cmd.arg("run");

            if let Some(config) = cfg {
                for (key, value) in config.environment_variables {
                    // arg must be processed as str or else we get extra quotes
                    let arg = format!("{}={}", key, value);
                    cmd.arg("-e").arg(arg);
                }
                if let Some(network) = config.network {
                    cmd.arg("--net").arg(network);
                }
            }
            cmd.arg(name.clone());
            cmd.stdout(::std::process::Stdio::piped());

            let mut child = cmd.spawn().unwrap();
            let secs = ::std::time::Duration::from_secs(20);

            let _status_code = match child.wait_timeout(secs).unwrap() {
                Some(status) => status.code(),
                None => {
                    stop_and_remove_container_by_image(name);
                    child.kill().unwrap();
                    child.wait().unwrap().code()
                }
            };

            let reader = ::std::io::BufReader::new(child.stdout.unwrap());
            reader
                .lines()
                .map(|line| line.unwrap())
                .collect::<Vec<_>>()
                .join("\n")
        }

        /// Builds a directory with default options
        /// Returns the randomly generated image name
        fn simple_build(path: &str) -> String {
            let raw_env = ::std::fs::read_to_string(format!("{}/test.env", path)).unwrap();
            let env = ::dotenv_parser::parse_dotenv(&raw_env).unwrap();
            let name = ::uuid::Uuid::new_v4().to_string();
            ::nixpacks::create_docker_image(
                path,
                env.get("TEST_DOCKER_ENVS").map(|envs| envs.split(", ").collect()).unwrap_or_default(),
                &::nixpacks::nixpacks::plan::config::GeneratePlanConfig {
                    pin_pkgs: !env.contains_key("TEST_DOCKER_NO_PIN_PKGS"),
                    custom_start_cmd: env.get("TEST_DOCKER_CUSTOM_START_CMD").map(|s| s.clone()),
                    custom_pkgs: env
                        .get("TEST_DOCKER_CUSTOM_PKGS")
                        .map(|pkgs| pkgs.split(',')
                        .map(|pkg| ::nixpacks::nixpacks::nix::pkg::Pkg::new(pkg)).collect())
                        .unwrap_or_default(),
                    ..::nixpacks::nixpacks::plan::config::GeneratePlanConfig::default()
                },
                &::nixpacks::nixpacks::builder::docker::DockerBuilderOptions {
                    name: Some(name.clone()),
                    quiet: true,
                    ..::nixpacks::nixpacks::builder::docker::DockerBuilderOptions::default()
                },
            )
            .unwrap();

            name
        }

        const POSTGRES_IMAGE: &str = "postgres";

        struct Network {
            name: String,
        }

        fn attach_container_to_network(network_name: String, container_name: String) {
            ::std::process::Command::new("docker")
                .arg("network")
                .arg("connect")
                .arg(network_name)
                .arg(container_name)
                .spawn()
                .unwrap()
                .wait()
                .context("Setting up network")
                .unwrap();
        }

        fn create_network() -> Network {
            let network_name = format!("test-net-{}", ::uuid::Uuid::new_v4());

            ::std::process::Command::new("docker")
                .arg("network")
                .arg("create")
                .arg(network_name.clone())
                .spawn()
                .unwrap()
                .wait()
                .context("Setting up network")
                .unwrap();

            Network { name: network_name }
        }

        fn remove_network(network_name: String) {
            ::std::process::Command::new("docker")
                .arg("network")
                .arg("rm")
                .arg(network_name)
                .spawn()
                .unwrap()
                .wait()
                .context("Tearing down network")
                .unwrap();
        }

        struct Container {
            name: String,
            config: Option<Config>,
        }

        fn run_postgres() -> Container {
            let mut docker_cmd = ::std::process::Command::new("docker");

            let hash = ::uuid::Uuid::new_v4().to_string();
            let container_name = format!("postgres-{}", hash);
            let password = hash;
            let port = "5432";
            // run
            docker_cmd.arg("run");

            // Set Needed Envvars
            docker_cmd
                .arg("-e")
                .arg(format!("POSTGRES_PASSWORD={}", &password));

            // Run detached
            docker_cmd.arg("-d");

            // attach name
            docker_cmd.arg("--name").arg(container_name.clone());

            // Assign image
            docker_cmd.arg(POSTGRES_IMAGE);

            // Run the command
            docker_cmd
                .spawn()
                .unwrap()
                .wait()
                .context("Building postgres")
                .unwrap();

            Container {
                name: container_name.clone(),
                config: Some(Config {
                    environment_variables: ::nixpacks::nixpacks::environment::EnvironmentVariables::from([
                        ("PGPORT".to_string(), port.to_string()),
                        ("PGUSER".to_string(), "postgres".to_string()),
                        ("PGDATABASE".to_string(), "postgres".to_string()),
                        ("PGPASSWORD".to_string(), password),
                        ("PGHOST".to_string(), container_name),
                    ]),
                    network: None,
                }),
            }
        }
    }];

    for example in examples {
        let test_name = format_ident!("{}", example.replace('-', "_"));
        let raw_env = fs::read_to_string(format!("./examples/{}/test.env", example))
            .unwrap()
            .replace("{{RANDOM_64_STRING}}", &gen_random_64_str());

        let env = parse_dotenv(&raw_env).unwrap();
        let expected_output = env.get("TEST_DOCKER_EXPECTED_OUTPUT").unwrap();
        let test = if env.get("TEST_DOCKER_WITH_POSTGRES").is_some() {
            quote! {
                #[test]
                fn #test_name() {
                    // Create the network
                    let n = create_network();
                    let network_name = n.name.clone();

                    // Create the postgres instance
                    let c = run_postgres();
                    let container_name = c.name.clone();

                    // Attach the postgres instance to the network
                    attach_container_to_network(n.name, container_name.clone());

                    // Build the Django example
                    let name = simple_build(&format!("./examples/{}", #example));

                    // Run the Django example on the attached network
                    let output = run_image(
                        name,
                        Some(Config {
                            environment_variables: c.config.unwrap().environment_variables,
                            network: Some(network_name.clone()),
                        }),
                    );

                    // Cleanup containers and networks
                    stop_and_remove_container(container_name);
                    remove_network(network_name);

                    assert!(output.contains(#expected_output));
                }
            }
        } else {
            quote! {
                #[test]
                fn #test_name() {
                    let name = simple_build(&format!("./examples/{}", #example));
                    let output = run_image(name, None);

                    assert!(output.contains(#expected_output));
                }
            }
        };

        tests.push(test);
    }

    tests
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}
