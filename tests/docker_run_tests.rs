use nixpacks::build;
use std::io::{BufRead, BufReader};
use std::{
    process::{Command, Stdio},
    thread, time,
};
use uuid::Uuid;

const TIMEOUT_SECONDS: i32 = 5;

fn get_container_ids_from_image(image: String) -> String {
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
        .unwrap();
}

fn remove_containers(container_id: &str) {
    Command::new("docker")
        .arg("rm")
        .arg(container_id)
        .spawn()
        .unwrap();
}

fn stop_and_remove_container(image: String) {
    let container_ids = get_container_ids_from_image(image);
    let container_id = container_ids.trim().split('\n').collect::<Vec<_>>()[0].to_string();

    stop_containers(&container_id);
    remove_containers(&container_id);
}

/// Runs an image with Docker and returns the output
/// The image is automatically stopped and removed after `TIMEOUT_SECONDS`
fn run_image(name: String) -> String {
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg(name.clone());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();

    let cloned_name = name.clone();

    let thread = thread::spawn(move || {
        for _ in 0..TIMEOUT_SECONDS {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }

            thread::sleep(time::Duration::from_secs(1));
        }

        stop_and_remove_container(name.clone());
        child.kill().unwrap();
    });

    let reader = BufReader::new(stdout);
    let output = reader
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    thread.join().unwrap();

    // Clean up container when done
    stop_and_remove_container(cloned_name);

    output
}

/// Builds a directory with default options
/// Returns the randomly generated image name
fn simple_build(path: &str) -> String {
    let name = Uuid::new_v4().to_string();
    build(
        path,
        Some(name.clone()),
        Vec::new(),
        None,
        None,
        true,
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
    )
    .unwrap();

    name
}

#[test]
fn test_node() {
    let name = simple_build("./examples/node");
    assert!(run_image(name).contains("Hello from Node"));
}

#[test]
fn test_node_custom_version() {
    let name = simple_build("./examples/node-custom-version");
    let output = run_image(name);
    assert!(output.contains("Node version: v14"));
}

#[test]
fn test_yarn_custom_version() {
    let name = simple_build("./examples/yarn-custom-node-version");
    let output = run_image(name);
    assert!(output.contains("Node version: v14"));
}

#[test]
fn test_yarn_prisma() {
    let name = simple_build("./examples/yarn-prisma");
    let output = run_image(name);
    assert!(output.contains("My post content"));
}

#[test]
fn test_python() {
    let name = simple_build("./examples/python");
    let output = run_image(name);
    assert!(output.contains("Hello from Python"));
}

#[test]
fn test_python_2() {
    let name = simple_build("./examples/python-2");
    let output = run_image(name);
    assert!(output.contains("Hello from Python 2"));
}

#[test]
fn test_rust_custom_version() {
    let name = simple_build("./examples/rust-custom-version");
    let output = run_image(name);
    assert!(output.contains("cargo 1.56.0"));
}

#[test]
fn test_go() {
    let name = simple_build("./examples/go");
    let output = run_image(name);
    assert!(output.contains("Hello from Go"));
}

#[test]
fn test_cowsay() {
    let name = Uuid::new_v4().to_string();
    build(
        "./examples/hello",
        Some(name.clone()),
        vec!["cowsay"],
        None,
        Some("./start.sh".to_string()),
        false,
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
    )
    .unwrap();
    let output = run_image(name);
    assert!(output.contains("Hello World"));
}
