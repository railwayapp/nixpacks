use nixpacks::{generate_build_plan, nixpacks::plan::generator::GeneratePlanOptions};
use std::env::consts::ARCH;

test_helper::generate_plan_tests!();

#[test]
fn test_custom_plan_path() {
    let plan = generate_build_plan(
        "./examples/custom-plan-path",
        Vec::new(),
        &GeneratePlanOptions {
            config_file: Some("custom-nixpacks.toml".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    assert_plan_snapshot!(plan);
}

#[test]
fn test_custom_rust_version() {
    let plan = simple_gen_plan("./examples/rust-custom-version");
    let setup = plan.get_phase("setup").unwrap().clone();
    let build = plan.get_phase("build").unwrap().clone();

    assert_eq!(
        build.cmds,
        Some(vec![
            format!("mkdir -p bin"),
            format!("cargo build --release --target {ARCH}-unknown-linux-musl"),
            format!("cp target/{ARCH}-unknown-linux-musl/release/rust-custom-version bin")
        ])
    );
    assert_eq!(
        setup
            .nix_pkgs
            .unwrap()
            .iter()
            .filter(|p| p.contains("1.56.0"))
            .count(),
        1
    );
}

#[test]
fn test_rust_rocket() {
    let plan = simple_gen_plan("./examples/rust-rocket");
    let build = plan.get_phase("build").unwrap();
    let start = plan.start_phase.clone().unwrap();

    assert_eq!(
        build.cmds,
        Some(vec![
            format!("mkdir -p bin"),
            format!("cargo build --release --target {ARCH}-unknown-linux-musl"),
            format!("cp target/{ARCH}-unknown-linux-musl/release/rocket bin")
        ])
    );
    assert!(start.cmd.is_some());
    assert_eq!(start.clone().cmd.unwrap(), "./bin/rocket".to_string());
    assert!(start.run_image.is_some());
}

#[test]
fn test_rust_rocket_no_musl() {
    let plan = generate_build_plan(
        "./examples/rust-rocket",
        vec!["NIXPACKS_NO_MUSL=1"],
        &GeneratePlanOptions::default(),
    )
    .unwrap();
    assert_plan_snapshot!(plan);
}

#[test]
fn test_rust_cargo_workspaces() {
    let plan = simple_gen_plan("./examples/rust-cargo-workspaces");
    let build = plan.get_phase("build").unwrap();

    assert_eq!(
        build.clone().cmds.unwrap()[1],
        format!("cargo build --release --package binary --target {ARCH}-unknown-linux-musl")
    );
}

#[test]
fn test_haskell_stack() {
    let plan = simple_gen_plan("./examples/haskell-stack");
    let install = plan.get_phase("install").unwrap();
    let build = plan.get_phase("build").unwrap();
    let start = plan.start_phase.clone().unwrap();

    assert_eq!(install.cmds, Some(vec!["stack setup".to_string()]));
    assert_eq!(build.cmds, Some(vec!["stack install".to_string()]));
    assert_eq!(
        start.cmd,
        Some("/root/.local/bin/haskell-stack-exe".to_string())
    );
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "i386"))]
#[test]
fn test_zig_gyro() {
    let plan = simple_gen_plan("./examples/zig-gyro");
    let install = plan.get_phase("install").unwrap().clone();
    let build = plan.get_phase("build").unwrap();
    let start = plan.start_phase.clone().unwrap();

    assert_eq!(
        build.cmds,
        Some(vec!["zig build -Drelease-safe=true".to_string()])
    );
    assert_eq!(start.cmd, Some("./zig-out/bin/zig-gyro".to_string()));
    assert!(install
        .cmds
        .unwrap()
        .get(0)
        .unwrap()
        .contains("mkdir /gyro"));
}

#[test]
fn test_node_turborepo_custom_app() {
    let plan = generate_build_plan(
        "./examples/node-turborepo",
        vec!["NIXPACKS_TURBO_APP_NAME=docs"],
        &GeneratePlanOptions::default(),
    )
    .unwrap();
    assert!(plan.start_phase.unwrap().cmd.unwrap().contains("docs"));
}
