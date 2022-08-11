use nixpacks::{
    generate_build_plan,
    nixpacks::{
        nix::pkg::Pkg,
        plan::{generator::GeneratePlanOptions, BuildPlan},
    },
};
use std::env::consts::ARCH;

macro_rules! assert_plan_snapshot {
    ($plan:expr) => {
        insta::assert_json_snapshot!($plan, {
            ".version" => "[version]",
            ".setup.archive" => "[archive]",
            ".setup.baseImage" => "[baseImage]",
        });
    }
}

fn simple_gen_plan(path: &str) -> BuildPlan {
    generate_build_plan(path, Vec::new(), &GeneratePlanOptions::default()).unwrap()
}

#[test]
fn test_node() {
    let plan = simple_gen_plan("./examples/node");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_no_lockfile() {
    let plan = simple_gen_plan("./examples/node-no-lockfile-canvas");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_npm_old_lockfile() {
    let plan = simple_gen_plan("./examples/node-npm-old-lockfile");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_npm() {
    let plan = simple_gen_plan("./examples/node-npm");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_no_scripts() {
    let plan = simple_gen_plan("./examples/node-no-scripts");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_custom_version() {
    let plan = simple_gen_plan("./examples/node-custom-version");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_monorepo() {
    let plan = simple_gen_plan("./examples/node-monorepo");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_yarn() {
    let plan = simple_gen_plan("./examples/node-yarn");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_yarn_berry() {
    let plan = simple_gen_plan("./examples/node-yarn-berry");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_yarn_custom_version() {
    let plan = simple_gen_plan("./examples/node-yarn-custom-node-version");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_pnpm() {
    let plan = simple_gen_plan("./examples/node-pnpm");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_bun() {
    let plan = simple_gen_plan("./examples/node-bun");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_bun_no_start() {
    let plan = simple_gen_plan("./examples/node-bun-no-start");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_bun_web_server() {
    let plan = simple_gen_plan("./examples/node-bun-no-start");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_pnpm_v7() {
    let plan = simple_gen_plan("./examples/node-pnpm-v7");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_pnpm_custom_version() {
    let plan = simple_gen_plan("./examples/node-pnpm-custom-node-version");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_go() {
    let plan = simple_gen_plan("./examples/go");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_go_cgo_enabled() {
    let plan = generate_build_plan(
        "./examples/go",
        vec!["CGO_ENABLED=1"],
        &GeneratePlanOptions::default(),
    )
    .unwrap();
    assert_plan_snapshot!(plan);
}

#[test]
fn test_go_mod() {
    let plan = simple_gen_plan("./examples/go-mod");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_go_custom_version() {
    let plan = simple_gen_plan("./examples/go-custom-version");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_deno() {
    let plan = simple_gen_plan("./examples/deno");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_deno_fresh() {
    let plan = simple_gen_plan("./examples/deno-fresh");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_csharp_api() {
    let plan = simple_gen_plan("./examples/csharp-api");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_fsharp_api() {
    let plan = simple_gen_plan("./examples/fsharp-api");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_csharp_cli() {
    let plan = simple_gen_plan("./examples/csharp-cli");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_procfile() {
    let plan = simple_gen_plan("./examples/procfile");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_custom_pkgs() {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions {
            custom_start_cmd: Some("./start.sh".to_string()),
            custom_pkgs: vec![Pkg::new("cowsay")],
            ..Default::default()
        },
    )
    .unwrap();
    assert_plan_snapshot!(plan);
}

#[test]
fn test_pin_archive() {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions {
            pin_pkgs: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_plan_snapshot!(plan);
}

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
pub fn test_python() {
    let plan = simple_gen_plan("./examples/python");
    assert_plan_snapshot!(plan);
}

#[test]
pub fn test_python_poetry() {
    let plan = simple_gen_plan("./examples/python-poetry");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_main_file() {
    let plan = simple_gen_plan("./examples/node-main-file");
    assert_plan_snapshot!(plan);
}

#[test]
pub fn test_python_setuptools() {
    let plan = simple_gen_plan("./examples/python-setuptools");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_node_main_file_doesnt_exist() {
    let plan = simple_gen_plan("./examples/node-main-file-not-exist");
    assert_plan_snapshot!(plan);
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

#[test]
fn test_crystal() {
    let plan = simple_gen_plan("./examples/crystal");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_overriding_environment_variables() {
    let plan = generate_build_plan(
        "./examples/node-variables",
        vec!["NODE_ENV=test"],
        &GeneratePlanOptions::default(),
    )
    .unwrap();
    assert_plan_snapshot!(plan);
}

#[test]
fn test_config_from_environment_variables() {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        vec![
            "NIXPACKS_PKGS=cowsay ripgrep",
            "NIXPACKS_INSTALL_CMD=install",
            "NIXPACKS_BUILD_CMD=build",
            "NIXPACKS_START_CMD=start",
            "NIXPACKS_RUN_IMAGE=alpine",
            "NIXPACKS_INSTALL_CACHE_DIRS=/tmp,foobar",
            "NIXPACKS_BUILD_CACHE_DIRS=/build,barbaz",
        ],
        &GeneratePlanOptions::default(),
    )
    .unwrap();

    assert_plan_snapshot!(plan);
}

#[test]
fn test_staticfile() {
    let plan = simple_gen_plan("./examples/staticfile");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_php_vanilla() {
    let plan = simple_gen_plan("./examples/php-vanilla");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_php_laravel() {
    let plan = simple_gen_plan("./examples/php-laravel");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_dart() {
    let plan = simple_gen_plan("./examples/dart");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_swift() {
    let plan = simple_gen_plan("./examples/swift");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_swift_vapor() {
    let plan = simple_gen_plan("./examples/swift-vapor");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_java_maven() {
    let plan = simple_gen_plan("./examples/java-maven");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_java_maven_wrapper() {
    let plan = simple_gen_plan("./examples/java-maven-wrapper");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_zig() {
    let plan = simple_gen_plan("./examples/zig");
    assert_plan_snapshot!(plan);
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

#[test]
fn test_ruby_rails() {
    let plan = simple_gen_plan("./examples/ruby-rails-postgres");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_ruby_sinatra() {
    let plan = simple_gen_plan("./examples/ruby-sinatra");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_clojure() {
    let plan = simple_gen_plan("./examples/clojure");
    assert_plan_snapshot!(plan);
}

#[test]
fn test_clojure_ring_app() {
    let plan = simple_gen_plan("./examples/clojure-ring-app");
    assert_plan_snapshot!(plan);
}
