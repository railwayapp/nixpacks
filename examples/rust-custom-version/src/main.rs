use std::process::Command;

fn main() {
    let mut cargo_version_command = Command::new("cargo").arg("version").spawn().unwrap();
    cargo_version_command.wait().unwrap();
}
