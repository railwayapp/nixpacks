use nixpacks::{generate_build_plan, nixpacks::plan::generator::GeneratePlanOptions};
use std::env::consts::ARCH;

test_helper::generate_plan_tests!();

#[test]
fn test_custom_rust_version() {
    let plan = simple_gen_plan("./examples/rust-custom-version");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            format!("cargo build --release --target {}-unknown-linux-musl", ARCH),
            format!(
                "cp target/{}-unknown-linux-musl/release/rust-custom-version rust-custom-version",
                ARCH
            )
        ])
    );
    assert_eq!(
        plan.setup
            .unwrap()
            .pkgs
            .iter()
            .filter(|p| p.name.contains("1.56.0"))
            .count(),
        1
    );
}

#[test]
fn test_rust_rocket() {
    let plan = simple_gen_plan("./examples/rust-rocket");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            format!("cargo build --release --target {}-unknown-linux-musl", ARCH),
            format!(
                "cp target/{}-unknown-linux-musl/release/rocket rocket",
                ARCH
            )
        ])
    );
    assert!(plan.start.clone().unwrap().cmd.is_some());
    assert_eq!(
        plan.start.clone().unwrap().cmd.unwrap(),
        "./rocket".to_string()
    );
    assert!(plan.start.unwrap().run_image.is_some());
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

    assert_eq!(
        plan.build.unwrap().cmds.unwrap()[0],
        format!(
            "cargo build --release --package binary --target {}-unknown-linux-musl",
            ARCH
        )
    );
}

#[test]
fn test_haskell_stack() {
    let plan = simple_gen_plan("./examples/haskell-stack");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["stack setup".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["stack install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("/root/.local/bin/haskell-stack-exe".to_string())
    );
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "i386"))]
#[test]
fn test_zig_gyro() {
    let plan = simple_gen_plan("./examples/zig-gyro");

    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["zig build -Drelease-safe=true".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./zig-out/bin/zig-gyro".to_string())
    );
    assert!(plan
        .install
        .unwrap()
        .cmds
        .unwrap()
        .get(0)
        .unwrap()
        .contains("mkdir /gyro"));
}
