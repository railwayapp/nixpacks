use anyhow::{Context, Result};
use nixpacks::{
    create_docker_image,
    nixpacks::{
        builder::docker::docker_helper::DockerHelper, builder::docker::DockerBuilderOptions,
        environment::EnvironmentVariables, plan::generator::GeneratePlanOptions,
    },
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::str;
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
        .arg(format!("ancestor={image}"))
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
        .arg("-v")
        .arg(container_id)
        .spawn()
        .unwrap()
        .wait()
        .context("Removing container")
        .unwrap();
}

fn remove_image(image_id: &str) {
    Command::new("docker")
        .arg("rmi")
        .arg(image_id)
        .spawn()
        .unwrap()
        .wait()
        .context("Removing image")
        .unwrap();
}

fn stop_and_remove_container_by_image(image: &str) {
    let container_ids = get_container_ids_from_image(image);
    container_ids
        .trim()
        .split('\n')
        .for_each(|container_id| stop_and_remove_container(container_id.to_string()));
    remove_image(image);
}

fn stop_and_remove_container(name: String) {
    stop_containers(&name);
    remove_containers(&name);
}

#[derive(Debug, Clone)]
struct Config {
    environment_variables: EnvironmentVariables,
    network: Option<String>,
}

/// Runs an image with Docker and returns the output
/// The image is automatically stopped and removed after `TIMEOUT_SECONDS`
async fn run_image(name: &str, cfg: Option<Config>) -> String {
    let mut cmd = Command::new("docker");
    cmd.arg("run");

    if let Some(config) = cfg {
        for (key, value) in config.environment_variables {
            // arg must be processed as str or else we get extra quotes
            let arg = format!("{key}={value}");
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
        Some(status) => {
            stop_and_remove_container_by_image(name);
            status.code()
        }
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

async fn build_with_hosts(path: &str, add_hosts: &[String], nginx_host: String) -> String {
    let name = Uuid::new_v4().to_string();
    let mut env: Vec<&str> = Vec::new();
    let env_var = format!("REMOTE_URL=http://{}", nginx_host);
    env.push(&*env_var);

    create_docker_image(
        path,
        env,
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            add_host: add_hosts.to_owned(),

            ..Default::default()
        },
    )
    .await
    .unwrap();

    name
}

async fn build_with_env(path: &str, env: Vec<&str>) -> anyhow::Result<()> {
    let name = Uuid::new_v4().to_string();

    create_docker_image(
        path,
        env,
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .await
}

/// Builds a directory with default options
/// Returns the randomly generated image name
async fn simple_build(path: &str) -> Result<String> {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        path,
        Vec::new(),
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .await?;

    Ok(name)
}

async fn build_with_build_time_env_vars(path: &str, env_vars: Vec<&str>) -> Result<String> {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        path,
        env_vars,
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .await?;

    Ok(name)
}

const POSTGRES_IMAGE: &str = "postgres";
const MYSQL_IMAGE: &str = "mysql";
const NGINX_IMAGE: &str = "nginx";

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
    let container_name = format!("postgres-{hash}");
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
                ("PGPASSWORD".to_string(), password.clone()),
                ("PGHOST".to_string(), container_name.clone()),
                (
                    "DATABASE_URL".to_string(),
                    format!("postgresql://postgres:{password}@{container_name}:{port}/postgres"),
                ),
            ]),
            network: None,
        }),
    }
}

fn run_mysql() -> Container {
    let mut docker_cmd = Command::new("docker");

    let hash = Uuid::new_v4().to_string();
    let container_name = format!("mysql-{hash}");
    let password = hash;
    // run
    docker_cmd.arg("run");

    // Set Needed Envvars
    docker_cmd
        .arg("-e")
        .arg(format!("MYSQL_ROOT_PASSWORD={}", &password))
        .arg("-e")
        .arg(format!("MYSQL_PASSWORD={}", &password))
        .arg("-e")
        .arg("MYSQL_USER=mysql")
        .arg("-e")
        .arg("MYSQL_DATABASE=mysql");

    // Run detached
    docker_cmd.arg("-d");

    // attach name
    docker_cmd.arg("--name").arg(container_name.clone());

    // Assign image
    docker_cmd.arg(MYSQL_IMAGE);

    // Run the command
    docker_cmd
        .spawn()
        .unwrap()
        .wait()
        .context("starting mysql")
        .unwrap();

    // MySQL starts listening for connections after it has initialised its default database
    // so wait until mysqladmin ping via TCP succeeds (or we timeout)
    let test_loop = format!("while ! mysqladmin ping --password={} -h localhost --port=3306 --protocol=TCP 2> /dev/null ; do echo 'waiting for mysql'; sleep 1; done", &password);
    let mut docker_exec_cmd = Command::new("docker");
    docker_exec_cmd
        .arg("exec")
        .arg(container_name.clone())
        .arg("/bin/sh")
        .arg("-c")
        .arg(test_loop);

    let mut child = docker_exec_cmd.spawn().unwrap();

    match child.wait_timeout(Duration::new(30, 0)).unwrap() {
        Some(_) => (),
        None => {
            // timed out waiting for mysql to start - cleanup the test process and the mysql container
            child.kill().unwrap();
            stop_and_remove_container(container_name);
            panic!("mysql failed to start");
        }
    };

    Container {
        name: container_name.clone(),
        config: Some(Config {
            environment_variables: EnvironmentVariables::from([
                ("DB_PORT".to_string(), "3306".to_string()),
                ("DB_USER".to_string(), "mysql".to_string()),
                ("DB_NAME".to_string(), "mysql".to_string()),
                ("DB_PASSWORD".to_string(), password),
                ("DB_HOST".to_string(), container_name),
            ]),
            network: None,
        }),
    }
}

fn run_nginx() -> Container {
    let mut docker_cmd = Command::new("docker");

    let hash = Uuid::new_v4().to_string();
    let container_name = format!("nginx-{hash}");

    // run
    docker_cmd.arg("run");

    // Run detached
    docker_cmd.arg("-d");

    // attach name
    docker_cmd.arg("--name").arg(container_name.clone());

    // Assign image
    docker_cmd.arg(NGINX_IMAGE);

    // Run the command
    docker_cmd
        .spawn()
        .unwrap()
        .wait()
        .context("Building nginx")
        .unwrap();

    Container {
        name: container_name.clone(),
        config: Some(Config {
            environment_variables: EnvironmentVariables::from([]),
            network: None,
        }),
    }
}

#[tokio::test]
async fn test_deno() {
    let name = simple_build("./examples/deno").await.unwrap();
    assert!(run_image(&name, None).await.contains("Hello from Deno"));
}

#[tokio::test]
async fn test_elixir_no_ecto() {
    let rand_64_str: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let secret_env = format!("SECRET_KEY_BASE={rand_64_str}");
    let name = build_with_build_time_env_vars(
        "./examples/elixir-phx-no-ecto",
        vec![&*secret_env, "MIX_ENV=prod"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None).await.contains("Hello from Phoenix"));
}

#[tokio::test]
async fn test_node() {
    let name = simple_build("./examples/node").await.unwrap();
    assert!(run_image(&name, None).await.contains("Hello from Node"));
}

#[tokio::test]
async fn test_node_nx_default_app() {
    let name = simple_build("./examples/node-nx").await.unwrap();
    assert!(run_image(&name, None)
        .await
        .contains("nx express app works"));
}

#[tokio::test]
async fn test_node_nx_next() {
    let name =
        build_with_build_time_env_vars("./examples/node-nx", vec!["NIXPACKS_NX_APP_NAME=next-app"])
            .await
            .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("ready - started server on 0.0.0.0:3000, url: http://localhost:3000"));
}

#[tokio::test]
async fn test_node_nx_start_command() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=start-command"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("nx express app works"));
}

#[tokio::test]
async fn test_node_nx_no_options() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=no-options"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("fake start command started!"));
}

#[tokio::test]
async fn test_node_nx_start_command_production() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=start-command-production"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("nx express app works"));
}

#[tokio::test]
async fn test_node_nx_node() {
    let name =
        build_with_build_time_env_vars("./examples/node-nx", vec!["NIXPACKS_NX_APP_NAME=node-app"])
            .await
            .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("Hello from node-app!"));
}

#[tokio::test]
async fn test_node_nx_express() {
    let name = build_with_build_time_env_vars(
        "./examples/node-nx",
        vec!["NIXPACKS_NX_APP_NAME=express-app"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("nx express app works"));
}

#[tokio::test]
async fn test_node_custom_version() {
    let name = simple_build("./examples/node-custom-version")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Node version: v22"));
}

#[tokio::test]
async fn test_node_canvas() {
    let name = simple_build("./examples/node-canvas").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Node canvas"));
}

#[tokio::test]
async fn test_node_moon_custom_build() {
    let name = build_with_build_time_env_vars(
        "./examples/node-moon-monorepo",
        vec![
            "NIXPACKS_MOON_APP_NAME=server",
            "NIXPACKS_MOON_BUILD_TASK=compile",
        ],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None).await.contains("Server listening at"));
}

#[tokio::test]
async fn test_node_moon_custom_start() {
    let name = build_with_build_time_env_vars(
        "./examples/node-moon-monorepo",
        vec![
            "NIXPACKS_MOON_APP_NAME=client",
            "NIXPACKS_MOON_START_TASK=serve",
        ],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None)
        .await
        .contains("ready - started server on 0.0.0.0:3000"));
}

#[tokio::test]
async fn test_pnpm_network_call_working_with_add_hosts() {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the nginx instance
    let c = run_nginx();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    let containers = DockerHelper::containers_in_network(&network_name);

    if containers.is_err() {
        panic!("Failed to fetch containers in network");
    }

    let mut vec_hosts = Vec::new();

    for (_, containerinfo) in containers.unwrap() {
        let add_host = format!(
            "{}:{}",
            containerinfo.name, containerinfo.ipv4_address_without_mask
        );
        vec_hosts.push(add_host);
    }

    // Build the basic example, a function that calls the database
    let name = build_with_hosts(
        "./examples/node-fetch-network",
        &vec_hosts,
        container_name.clone(),
    )
    .await;

    // Run the example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Fetched data: OK"));
}

#[tokio::test]
async fn test_pnpm_network_call_should_not_work_without_hosts() {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the nginx instance
    let c = run_nginx();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    let containers = DockerHelper::containers_in_network(&network_name);

    if containers.is_err() {
        panic!("Failed to fetch containers in network");
    }

    let mut vec_hosts = Vec::new();

    for (_, container_info) in containers.unwrap() {
        let add_host = format!(
            "{}:{}",
            container_info.name, container_info.ipv4_address_without_mask
        );
        vec_hosts.push(add_host);
    }

    let mut env: Vec<&str> = Vec::new();
    let env_var = format!("REMOTE_URL=http://{}", container_name);
    env.push(&*env_var);

    // Build the basic example, a function that calls the database
    let build_result = build_with_env("./examples/node-fetch-network", env).await;

    assert!(build_result.is_err());

    stop_and_remove_container(container_name);
    remove_network(network_name);
}

#[tokio::test]
async fn test_prisma_postgres() -> Result<()> {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the basic example, a function that calls the database
    let name = match simple_build("./examples/node-prisma-postgres").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    // Run the example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("My post content"));
    Ok(())
}

#[tokio::test]
async fn test_bun_prisma_postgres() -> Result<()> {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the basic example, a function that calls the database
    let name = match simple_build("./examples/node-bun-prisma").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    // Run the example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    println!("OUTPUT = {output}");

    assert!(output.contains("All migrations have been successfully applied"));
    Ok(())
}

#[tokio::test]
async fn test_prisma_postgres_npm_v9() -> Result<()> {
    // This test is similar to the prisma_postgres test, but uses npm 9
    // This is because npm 9 handles node-gyp differently, and we want to make
    // sure that we can still build node-gyp packages with npm 9

    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the basic example, a function that calls the database
    let name = match simple_build("./examples/node-prisma-postgres-npm-v9").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    // Run the example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("My post content"));
    Ok(())
}

#[tokio::test]
async fn test_yarn_custom_version() {
    let name = simple_build("./examples/node-yarn-custom-node-version")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Node version: v16"));
}

#[tokio::test]
async fn test_node_turborepo() {
    let name = build_with_build_time_env_vars(
        "./examples/node-turborepo",
        vec!["NIXPACKS_TURBO_APP_NAME=web"],
    )
    .await
    .unwrap();

    assert!(run_image(&name, None).await.contains("> next start"));
}

#[tokio::test]
async fn test_yarn_berry() {
    let name = simple_build("./examples/node-yarn-berry").await.unwrap();
    let output = run_image(&name, None).await;

    assert!(output.contains("Hello from Yarn v2+"));
}

#[tokio::test]
async fn test_yarn_prisma() {
    let name = simple_build("./examples/node-yarn-prisma").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("My post content"));
}

#[tokio::test]
async fn test_pnpm_v8() {
    let name = simple_build("./examples/node-pnpm-v8").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from PNPM 8"));
}

#[tokio::test]
async fn test_pnpm_v9() {
    let name = simple_build("./examples/node-pnpm-v9").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from PNPM 9"));
}

#[tokio::test]
async fn test_pnpm_corepack() {
    let name = simple_build("./examples/node-pnpm-corepack").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from PNPM 9"));
}

#[tokio::test]
async fn test_bun() {
    let name = simple_build("./examples/node-bun").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Bun"));
}

#[tokio::test]
async fn test_bun_web_server() {
    let name = simple_build("./examples/node-bun-web-server")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from a Bun web server!"));
}

#[tokio::test]
async fn test_puppeteer() {
    let name = simple_build("./examples/node-puppeteer").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from puppeteer"));
}

#[tokio::test]
async fn test_csharp() {
    let name = simple_build("./examples/csharp-cli").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello world from C#"));
}

#[tokio::test]
async fn test_fsharp() {
    let name = simple_build("./examples/fsharp-cli").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello world from F#"));
}

#[tokio::test]
async fn test_python() {
    let name = simple_build("./examples/python").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python"));
}

#[tokio::test]
async fn test_python_pipfile() {
    let name = simple_build("./examples/python-pipfile").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Data fetched successfully!"));
}

#[tokio::test]
async fn test_python_procfile() {
    let name = simple_build("./examples/python-procfile").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python"));
}

#[tokio::test]
async fn test_python_2() {
    let name = simple_build("./examples/python-2").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python 2"));
}

#[tokio::test]
async fn test_python_asdf_poetry() {
    let name = simple_build("./examples/python-asdf-poetry").await.unwrap();
    let output = run_image(&name, None).await;

    assert!(output.contains("3.12.7"), "{}", output);
    assert!(output.contains("Poetry (version 1.8.2)"), "{}", output);
}

#[tokio::test]
async fn test_python_psycopg2() -> Result<()> {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    let name = match simple_build("./examples/python-psycopg2").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    println!("OUTPUT = {}", output);

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("User inserted successfully with ID"));
    Ok(())
}

#[tokio::test]
async fn test_django() -> Result<()> {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the Django example
    let name = match simple_build("./examples/python-django").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    // Run the Django example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Running migrations"));
    Ok(())
}

#[tokio::test]
async fn test_django_mysql() -> Result<()> {
    let n = create_network();
    let network_name = n.name.clone();

    let c = run_mysql();
    let container_name = c.name.clone();

    attach_container_to_network(n.name, container_name.clone());

    let name = match simple_build("./examples/python-django-mysql").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Running migrations"));
    Ok(())
}

#[tokio::test]
async fn test_python_poetry() {
    let name = simple_build("./examples/python-poetry").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python-Poetry"));
}

#[tokio::test]
async fn test_python_uv() {
    let name = simple_build("./examples/python-uv").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python-Uv"));
}

#[tokio::test]
async fn test_python_pdm() {
    let name = simple_build("./examples/python-pdm").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python-PDM"));
}

#[tokio::test]
async fn test_python_numpy() {
    let name = simple_build("./examples/python-numpy").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Python numpy and pandas"));
}

#[tokio::test]
async fn test_python_postgres() {
    let name = simple_build("./examples/python-postgres").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("psycopg2"));
}

#[tokio::test]
async fn test_rust_custom_version() {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        "./examples/rust-custom-version",
        vec!["NIXPACKS_NO_MUSL=1"],
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let output = run_image(&name, None).await;
    assert!(output.contains("cargo 1.56.0"));
}

#[tokio::test]
async fn test_rust_toolchain_file() {
    let name = simple_build("./examples/rust-custom-toolchain")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("cargo 1.82.0-nightly"));
}

#[tokio::test]
async fn test_rust_ring() {
    let name = simple_build("./examples/rust-ring").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from rust"));
}

#[tokio::test]
async fn test_rust_openssl() {
    let name = simple_build("./examples/rust-openssl").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Rust openssl!"));
}

#[tokio::test]
async fn test_rust_cargo_workspaces() {
    let name = simple_build("./examples/rust-cargo-workspaces")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from rust"));
}

#[tokio::test]
async fn test_rust_cargo_workspaces_glob() {
    let name = simple_build("./examples/rust-cargo-workspaces-glob")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from rust"));
}

#[tokio::test]
async fn test_rust_multiple_bins() {
    let name = simple_build("./examples/rust-multiple-bins").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Bin 1"));
}

#[tokio::test]
async fn test_gleam_basic() {
    let name = simple_build("./examples/basic_gleam").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Gleam!"));
}

#[tokio::test]
async fn test_go() {
    let name = simple_build("./examples/go").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Go"));
}

#[tokio::test]
async fn test_go_custom_version() {
    let name = simple_build("./examples/go-custom-version").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from go1.18"));
}

#[tokio::test]
async fn test_haskell_stack() {
    let name = simple_build("./examples/haskell-stack").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Haskell"));
}

#[tokio::test]
async fn test_crystal() {
    let name = simple_build("./examples/crystal").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Crystal"));
}

#[tokio::test]
async fn test_cowsay() {
    let name = Uuid::new_v4().to_string();
    create_docker_image(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello World"));
}

// This test is intentionally written to fail
#[tokio::test]
async fn test_docker_host() {
    let name = Uuid::new_v4().to_string();
    let result = create_docker_image(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions::default(),
        &DockerBuilderOptions {
            name: Some(name.clone()),
            quiet: true,
            docker_host: Some("tcp://0.0.0.0:2375".to_string()),
            docker_tls_verify: Some("0".to_string()),
            ..Default::default()
        },
    )
    .await;

    // Expect the creation of the Docker image to fail
    assert!(result.is_err());

    let output = run_image(&name, None).await;
    assert!(!output.contains("Hello World"));
}

#[tokio::test]
async fn test_staticfile() {
    let name = simple_build("./examples/staticfile").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("start worker process"));
}

#[tokio::test]
async fn test_swift() {
    let name = simple_build("./examples/swift").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from swift"));
}

#[tokio::test]
async fn test_dart() {
    let name = simple_build("./examples/dart").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Dart"));
}

#[tokio::test]
async fn test_java_gradle_8() {
    let name = simple_build("./examples/java-gradle-8").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Gradle 8"));
    assert!(output.contains("Hello from Java Gradle"));
}

#[tokio::test]
async fn test_java_maven() {
    let name = simple_build("./examples/java-maven").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Built with Spring Boot"));
}

#[tokio::test]
async fn test_java_spring_boot_3() {
    let name = simple_build("./examples/java-spring-boot-3").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Started HelloSpringApplication"));
}

#[tokio::test]
async fn test_java_spring_boot_2() {
    let name = simple_build("./examples/java-spring-boot-2").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Started HelloSpringApplication"));
}

#[tokio::test]
async fn test_java_spring_boot_1() {
    let name = simple_build("./examples/java-spring-boot-1").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Started HelloSpringApplication"));
}

#[tokio::test]
async fn test_php_vanilla() {
    let name = simple_build("./examples/php-vanilla").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Server starting on port 80"));
}

#[tokio::test]
async fn test_scala_sbt() {
    let name = simple_build("./examples/scala-sbt").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("I was compiled by Scala 3"));
}

#[tokio::test]
async fn test_zig() {
    let name = simple_build("./examples/zig").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Zig"));
}

#[tokio::test]
async fn test_ruby_2() {
    let name = simple_build("./examples/ruby-2").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Ruby 2"));
}

#[tokio::test]
async fn test_ruby_3() {
    let name = simple_build("./examples/ruby-3").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Ruby 3! YJIT is enabled."));
}

#[tokio::test]
async fn test_ruby_sinatra() {
    let name = simple_build("./examples/ruby-sinatra/").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Sinatra"));
}

#[tokio::test]
async fn test_ruby_node() {
    let name = simple_build("./examples/ruby-with-node/").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello from Ruby with Node"));
}

#[tokio::test]
async fn test_ruby_execjs() {
    let name = simple_build("./examples/ruby-execjs/").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("HELLO FROM EXECJS"));
}

#[tokio::test]
async fn test_ruby_local_deps() {
    let name = simple_build("./examples/ruby-local-deps/").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Hello world from Local lib"));
}

#[tokio::test]
async fn test_ruby_rails() -> Result<()> {
    // Create the network
    let n = create_network();
    let network_name = n.name.clone();

    // Create the postgres instance
    let c = run_postgres();
    let container_name = c.name.clone();

    // Attach the postgres instance to the network
    attach_container_to_network(n.name, container_name.clone());

    // Build the Rails example
    let name = match simple_build("./examples/ruby-rails-postgres").await {
        Ok(name) => name,
        Err(err) => {
            // Cleanup containers and networks, and then error
            stop_and_remove_container(container_name);
            remove_network(network_name);
            return Err(err);
        }
    };

    // Run the Rails example on the attached network
    let output = run_image(
        &name,
        Some(Config {
            environment_variables: c.config.unwrap().environment_variables,
            network: Some(network_name.clone()),
        }),
    )
    .await;

    // Cleanup containers and networks
    stop_and_remove_container(container_name);
    remove_network(network_name);

    assert!(output.contains("Rails 7"));
    Ok(())
}

#[tokio::test]
async fn test_ruby_rails_api_app() {
    let name = simple_build("./examples/ruby-rails-api-app").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Rails 7"));
}

#[tokio::test]
async fn test_clojure() {
    let name = simple_build("./examples/clojure").await.unwrap();
    let output = run_image(&name, None).await;
    assert_eq!(output, "Hello, World From Clojure!");
}

#[tokio::test]
async fn test_clojure_luminus() {
    let name = simple_build("./examples/clojure-luminus").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("clojure-luminus started successfully"));
}

#[tokio::test]
async fn test_clojure_ring_app() {
    let name = simple_build("./examples/clojure-ring-app").await.unwrap();
    let output = run_image(&name, None).await;
    assert_eq!(output, "Started server on port 3000");
}

#[tokio::test]
async fn test_clojure_tools_build() {
    let name = simple_build("./examples/clojure-tools-build")
        .await
        .unwrap();
    let output = run_image(&name, None).await;
    assert_eq!(output, "Hello, World From Clojure!");
}

#[tokio::test]
async fn test_cobol() {
    let name = simple_build("./examples/cobol").await.unwrap();
    let output = run_image(&name, None).await;
    assert_eq!(output, "Hello from cobol! index");
}

#[tokio::test]
async fn test_cobol_src_index() {
    let name = simple_build("./examples/cobol-src").await.unwrap();
    let output = run_image(&name, None).await;
    assert_eq!(output, "Hello from cobol! src-index");
}

#[tokio::test]
async fn test_cobol_my_app() {
    let name =
        build_with_build_time_env_vars("./examples/cobol", vec!["NIXPACKS_COBOL_APP_NAME=my-app"])
            .await
            .unwrap();

    assert_eq!(run_image(&name, None).await, "Hello from cobol! my-app");
}

#[tokio::test]
async fn test_cobol_src_my_app() {
    let name = build_with_build_time_env_vars(
        "./examples/cobol-src",
        vec!["NIXPACKS_COBOL_APP_NAME=my-app"],
    )
    .await
    .unwrap();

    assert_eq!(run_image(&name, None).await, "Hello from cobol! src-my-app");
}

#[tokio::test]
async fn test_cobol_free() {
    let name = build_with_build_time_env_vars(
        "./examples/cobol",
        vec![
            "NIXPACKS_COBOL_APP_NAME=cobol-free",
            "NIXPACKS_COBOL_COMPILE_ARGS=-free -x -o",
        ],
    )
    .await
    .unwrap();

    assert_eq!(run_image(&name, None).await, "Hello from cobol! cobol-free");
}

#[tokio::test]
async fn test_cobol_no_index() {
    let name = simple_build("./examples/cobol-no-index").await.unwrap();

    assert_eq!(
        run_image(&name, None).await,
        "Hello from cobol! cobol-no-index"
    );
}

#[tokio::test]
async fn test_nested_directory() {
    let name = simple_build("./examples/nested").await.unwrap();
    assert!(run_image(&name, None).await.contains("Nested directories!"));
}

#[tokio::test]
async fn test_ffmpeg() {
    let name = simple_build("./examples/apt-ffmpeg").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("ffmpeg version"));
}

#[tokio::test]
async fn test_node_python() {
    let name = simple_build("./examples/node-python").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("Node"));
    assert!(output.contains("Python"));
}

#[tokio::test]
async fn test_config_toml_file() {
    let name = simple_build("./examples/config-toml-file").await.unwrap();
    let output = run_image(&name, None).await;
    assert!(output.contains("hey there"));
}

#[tokio::test]
async fn test_scheme() {
    let name = simple_build("./examples/scheme").await.unwrap();
    assert!(run_image(&name, None).await.contains("Hello from Scheme!"));
}
