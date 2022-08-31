use anyhow::Context;
use nixpacks::{
    create_docker_image,
    nixpacks::{
        builder::docker::DockerBuilderOptions, environment::EnvironmentVariables, nix::pkg::Pkg,
        plan::config::GeneratePlanConfig,
    },
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;
use uuid::Uuid;
use wait_timeout::ChildExt;

use rand::thread_rng;
use rand::{distributions::Alphanumeric, Rng};

fn get_container_ids_from_image(image: &str) -> String {
    let output = Command::new("docker")
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
    Command::new("docker")
        .arg("stop")
        .arg(container_id)
        .spawn()
        .unwrap()
        .wait()
        .context("Stopping container")
        .unwrap();
}

fn remove_containers(container_id: &str) {
    Command::new("docker")
        .arg("rm")
        .arg(container_id)
        .spawn()
        .unwrap()
        .wait()
        .context("Removing container")
        .unwrap();
}

fn stop_and_remove_container_by_image(image: &str) {
    let container_ids = get_container_ids_from_image(image);
    let container_id = container_ids.trim().split('\n').collect::<Vec<_>>()[0].to_string();

    stop_and_remove_container(container_id);
}

fn stop_and_remove_container(name: String) {
    stop_containers(&name);
    remove_containers(&name);
}

struct Config {
    environment_variables: EnvironmentVariables,
    network: Option<String>,
}
/// Runs an image with Docker and returns the output
/// The image is automatically stopped and removed after `TIMEOUT_SECONDS`
fn run_image(name: &str, cfg: Option<Config>) -> String {
    let mut cmd = Command::new("docker");
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
    cmd.arg(name);
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().unwrap();
    let secs = Duration::from_secs(20);

    let _status_code = match child.wait_timeout(secs).unwrap() {
        Some(status) => status.code(),
        None => {
            stop_and_remove_container_by_image(name);
            child.wait().unwrap().code()
        }
    };

    let reader = BufReader::new(child.stdout.unwrap());
    reader
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Builds a directory with default options
/// Returns the randomly generated image name
fn simple_build(path: &str) -> String {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        path,
        Vec::new(),
        &GeneratePlanConfig {
            pin_pkgs: true,
            ..Default::default()
        },
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .unwrap();

    name
}

fn build_with_build_time_env_vars(path: &str, env_vars: Vec<&str>) -> String {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        path,
        env_vars,
        &GeneratePlanConfig {
            pin_pkgs: true,
            ..Default::default()
        },
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
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
    Command::new("docker")
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
    let network_name = format!("test-net-{}", Uuid::new_v4());

    Command::new("docker")
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
    Command::new("docker")
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
    let mut docker_cmd = Command::new("docker");

    let hash = Uuid::new_v4().to_string();
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
            environment_variables: EnvironmentVariables::from([
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

#[test]
fn test_deno() {
    let name = simple_build("./examples/deno");
    assert!(run_image(&name, None).contains("Hello from Deno"));
}

#[test]
fn test_elixir_no_ecto() {
    let rand_64_str: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let secret_env = format!("SECRET_KEY_BASE={}", rand_64_str);
    let name = build_with_build_time_env_vars(
        "./examples/elixir_no_ecto",
        vec![&*secret_env, "MIX_ENV=prod"],
    );

    assert!(run_image(&name, None).contains("Hello from Phoenix"));
}

#[test]
fn test_node() {
    let name = simple_build("./examples/node");
    assert!(run_image(&name, None).contains("Hello from Node"));
}

#[test]
fn test_node_nx_default_app() {
    let name = simple_build("./examples/node-nx");
    assert!(run_image(&name, None).contains("nx express app works"));
}

#[test]
fn test_node_nx_next() {
    let name =
        build_with_build_time_env_vars("./examples/node-nx", vec!["NIXPACKS_NX_APP_NAME=next-app"]);

    assert!(run_image(&name, None)
        .contains("ready - started server on 0.0.0.0:3000, url: http://localhost:3000"));
}

#[test]
fn test_node_nx_start_command() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=start-command"],
    );

    assert!(run_image(&name, None).contains("nx express app works"));
}

#[test]
fn test_node_nx_start_command_production() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=start-command-production"],
    );

    assert!(run_image(&name, None).contains("nx express app works"));
}

#[test]
fn test_node_nx_node() {
    let name =
        build_with_build_time_env_vars("./examples/node-nx", vec!["NIXPACKS_NX_APP_NAME=node-app"]);

    assert!(run_image(&name, None).contains("Hello from node-app!"));
}

#[test]
fn test_node_nx_express() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=express-app"],
    );

    assert!(run_image(&name, None).contains("nx express app works"));
}

#[test]
fn test_node_custom_version() {
    let name = simple_build("./examples/node-custom-version");
    let output = run_image(&name, None);
    assert!(output.contains("Node version: v18"));
}

#[test]
fn test_node_canvas() {
    let name = simple_build("./examples/node-canvas");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Node canvas"));
}

#[test]
fn test_yarn_custom_version() {
    let name = simple_build("./examples/node-yarn-custom-node-version");
    let output = run_image(&name, None);
    assert!(output.contains("Node version: v14"));
}

#[test]
fn test_yarn_berry() {
    let name = simple_build("./examples/node-yarn-berry");
    let output = run_image(&name, None);

    assert!(output.contains("Hello from Yarn v2+"));
}

#[test]
fn test_yarn_prisma() {
    let name = simple_build("./examples/node-yarn-prisma");
    let output = run_image(&name, None);
    assert!(output.contains("My post content"));
}

#[test]
fn test_pnpm() {
    let name = simple_build("./examples/node-pnpm");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from PNPM"));
}

#[test]
fn test_bun() {
    let name = simple_build("./examples/node-bun");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Bun"));
}

#[test]
fn test_bun_web_server() {
    let name = simple_build("./examples/node-bun-web-server");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from a Bun web server!"));
}

#[test]
fn test_pnpm_custom_version() {
    let name = simple_build("./examples/node-pnpm-custom-node-version");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from PNPM"));
}

#[test]
fn test_puppeteer() {
    let name = simple_build("./examples/node-puppeteer");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from puppeteer"));
}

#[test]
fn test_csharp() {
    let name = simple_build("./examples/csharp-cli");
    let output = run_image(&name, None);
    assert!(output.contains("Hello world from C#"));
}

#[test]
fn test_fsharp() {
    let name = simple_build("./examples/fsharp-cli");
    let output = run_image(&name, None);
    assert!(output.contains("Hello world from F#"));
}

#[test]
fn test_python() {
    let name = simple_build("./examples/python");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Python"));
}

#[test]
fn test_python_procfile() {
    let name = simple_build("./examples/python-procfile");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Python"));
}

#[test]
fn test_python_2() {
    let name = simple_build("./examples/python-2");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Python 2"));
}

#[test]
fn test_django() {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the Django example
    let name = simple_build("./examples/python-django");

    // Run the Django example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    );

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Running migrations"));
}

#[test]
fn test_python_poetry() {
    let name = simple_build("./examples/python-poetry");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Python-Poetry"));
}

#[test]
fn test_python_numpy() {
    let name = simple_build("./examples/python-numpy");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Python numpy and pandas"));
}

#[test]
fn test_rust_custom_version() {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        "./examples/rust-custom-version",
        vec!["NIXPACKS_NO_MUSL=1"],
        &GeneratePlanConfig {
            pin_pkgs: true,
            ..Default::default()
        },
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .unwrap();

    let output = run_image(&name, None);
    assert!(output.contains("cargo 1.56.0"));
}

#[test]
fn test_rust_ring() {
    let name = simple_build("./examples/rust-ring");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from rust"));
}

#[test]
fn test_rust_openssl() {
    let name = simple_build("./examples/rust-openssl");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Rust openssl!"));
}

#[test]
fn test_rust_cargo_workspaces() {
    let name = simple_build("./examples/rust-cargo-workspaces");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from rust"));
}

#[test]
fn test_rust_cargo_workspaces_glob() {
    let name = simple_build("./examples/rust-cargo-workspaces-glob");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from rust"));
}

#[test]
fn test_go() {
    let name = simple_build("./examples/go");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Go"));
}

#[test]
fn test_go_custom_version() {
    let name = simple_build("./examples/go-custom-version");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from go1.18"));
}

#[test]
fn test_haskell_stack() {
    let name = simple_build("./examples/haskell-stack");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Haskell"));
}

#[test]
fn test_crystal() {
    let name = simple_build("./examples/crystal");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Crystal"));
}

#[test]
fn test_cowsay() {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanConfig {
            pin_pkgs: true,
            custom_start_cmd: Some("./start.sh".to_string()),
            custom_pkgs: vec![Pkg::new("cowsay")],
            ..Default::default()
        },
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .unwrap();
    let output = run_image(&name, None);
    assert!(output.contains("Hello World"));
}

#[test]
fn test_staticfile() {
    let name = simple_build("./examples/staticfile");
    let output = run_image(&name, None);
    assert!(output.contains("start worker process"));
}

#[test]
fn test_swift() {
    let name = Uuid::new_v4().to_string();

    create_docker_image(
        "./examples/swift",
        Vec::new(),
        &GeneratePlanConfig {
            pin_pkgs: false,
            ..Default::default()
        },
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .unwrap();

    let output = run_image(&name, None);
    assert!(output.contains("Hello from swift"));
}

#[test]
fn test_dart() {
    let name = simple_build("./examples/dart");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Dart"));
}

#[test]
fn test_java_maven() {
    let name = simple_build("./examples/java-maven");
    let output = run_image(&name, None);
    assert!(output.contains("Built with Spring Boot"));
}

#[test]
fn test_zig() {
    let name = simple_build("./examples/zig");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Zig"));
}

#[test]
fn test_zig_gyro() {
    let name = simple_build("./examples/zig-gyro");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Zig"));
    assert!(output.contains("The URI scheme of GitHub is https."));
}

#[test]
fn test_ruby_sinatra() {
    let name = simple_build("./examples/ruby-sinatra/");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Sinatra"));
}

#[test]
fn test_ruby_node() {
    let name = simple_build("./examples/ruby-with-node/");
    let output = run_image(&name, None);
    assert!(output.contains("Hello from Ruby with Node"));
}

#[test]
fn test_ruby_local_deps() {
    let name = simple_build("./examples/ruby-local-deps/");
    let output = run_image(&name, None);
    assert!(output.contains("Hello world from Local lib"));
}

#[test]
fn test_ruby_rails() {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the Django example
    let name = simple_build("./examples/ruby-rails-postgres");

    // Run the Rails example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    );

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Rails 7"));
}

#[test]
fn test_clojure() {
    let name = simple_build("./examples/clojure");
    let output = run_image(&name, None);
    assert_eq!(output, "Hello, World From Clojure!");
}

#[test]
fn test_clojure_ring_app() {
    let name = simple_build("./examples/clojure-ring-app");
    let output = run_image(&name, None);
    assert_eq!(output, "Started server on port 3000");
}
