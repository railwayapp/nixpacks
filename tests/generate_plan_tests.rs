use nixpacks::{
    generate_build_plan,
    nixpacks::{
        nix::pkg::Pkg,
        plan::{generator::GeneratePlanOptions, BuildPlan},
    },
};
use std::env::consts::ARCH;

fn simple_gen_plan(path: &str) -> BuildPlan {
    generate_build_plan(path, Vec::new(), &GeneratePlanOptions::default()).unwrap()
}

macro_rules! gen_test {
    ($($name:ident $(,path = $path:literal)? $(,envs = $envs:expr)? $(,options = $options:expr)? $(,)?);*) => ($(
        #[test]
        #[allow(unused_variables)]
        fn $name() {
            let path = ::const_str::replace!(concat!("./examples/", stringify!($name)), "_", "-");
            let envs = Vec::<&str>::new();
            let options = &GeneratePlanOptions::default();

            $(let path = $path;)?
            $(let envs = $envs;)?
            $(let options = $options;)?

            let plan = generate_build_plan(path, envs, options).unwrap();
            ::insta::assert_json_snapshot!(plan);
        }
    )*);
}

gen_test! {
    node;
    node_no_lockfile, path = "./examples/node-no-lockfile-canvas";
    node_npm_old_lockfile;
    npm, path = "./examples/node-npm";
    node_no_scripts;
    node_custom_version;
    node_monorepo;
    yarn, path = "./examples/node-yarn";
    yarn_berry, path = "./examples/node-yarn-berry";
    yarn_custom_version, path = "./examples/node-yarn-custom-node-version";
    pnpm, path = "./examples/node-pnpm";
    bun, path = "./examples/node-bun";
    bun_no_start, path = "./examples/node-bun-no-start";
    bun_web_server, path = "./examples/node-bun-web-server";
    pnpm_v7, path = "./examples/node-pnpm-v7";
    pnpm_custom_version, path = "./examples/node-pnpm-custom-node-version";
    go;
    go_cgo_enabled, path = "./examples/go", envs = vec!["CGO_ENABLED=1"];
    go_mod;
    go_custom_version;
    deno;
    deno_fresh;
    csharp_api;
    fsharp_api;
    csharp_cli;
    procfile;
    custom_pkgs, path = "./examples/shell-hello", options = &GeneratePlanOptions {
        custom_start_cmd: Some("./start.sh".to_string()),
        custom_pkgs: vec![Pkg::new("cowsay")],
        ..Default::default()
    };
    pin_archive, path = "./examples/shell-hello", options = &GeneratePlanOptions {
        pin_pkgs: true,
        ..Default::default()
    };
    rust_rocket_no_musl, path = "./examples/rust-rocket", envs = vec!["NIXPACKS_NO_MUSL=1"];
    python;
    python_poetry;
    node_main_file;
    python_setuptools;
    node_main_file_doesnt_exist, path = "./examples/node-main-file-not-exist";
    crystal;
    overriding_environment_variables, path = "./examples/node-variables", envs = vec!["NODE_ENV=test"];
    config_from_environment_variables, path = "./examples/shell-hello", envs = vec![
        "NIXPACKS_PKGS=cowsay ripgrep",
        "NIXPACKS_INSTALL_CMD=install",
        "NIXPACKS_BUILD_CMD=build",
        "NIXPACKS_START_CMD=start",
        "NIXPACKS_RUN_IMAGE=alpine",
        "NIXPACKS_INSTALL_CACHE_DIRS=/tmp,foobar",
        "NIXPACKS_BUILD_CACHE_DIRS=/build,barbaz",
    ];
    staticfile;
    php_vanilla;
    php_laravel;
    dart;
    swift;
    swift_vapor;
    java_maven;
    java_maven_wrapper;
    zig;
    ruby_rails, path = "./examples/ruby-rails-postgres";
    ruby_sinatra;
    clojure;
    clojure_ring_app
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
fn rust_rocket() {
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
fn haskell_stack() {
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
fn zig_gyro() {
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
