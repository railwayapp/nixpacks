use anyhow::Result;
use nixpacks::{
    generate_build_plan,
    nixpacks::{
        nix::pkg::Pkg,
        plan::{generator::GeneratePlanOptions, BuildPlan},
    },
};

fn simple_gen_plan(path: &str) -> BuildPlan {
    generate_build_plan(path, Vec::new(), &GeneratePlanOptions::default()).unwrap()
}

#[test]
fn test_node() -> Result<()> {
    let plan = simple_gen_plan("./examples/node");
    assert_eq!(plan.install.unwrap().cmd, Some("npm ci".to_string()));
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_node_no_lockfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-no-lockfile");
    assert_eq!(plan.install.unwrap().cmd, Some("npm i".to_string()));
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_npm() -> Result<()> {
    let plan = simple_gen_plan("./examples/npm");
    assert_eq!(plan.build.unwrap().cmd, Some("npm run build".to_string()));
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_node_no_scripts() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-no-scripts");
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_node_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-custom-version");
    assert_eq!(plan.setup.unwrap().pkgs, vec![Pkg::new("nodejs-18_x")]);

    Ok(())
}

#[test]
fn test_yarn() -> Result<()> {
    let plan = simple_gen_plan("./examples/yarn");
    assert_eq!(plan.build.unwrap().cmd, Some("yarn run build".to_string()));
    assert_eq!(plan.start.unwrap().cmd, Some("yarn run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_yarn_berry() -> Result<()> {
    let plan = simple_gen_plan("./examples/yarn-berry");
    assert_eq!(
        plan.install.unwrap().cmd,
        Some("yarn set version berry && yarn install --immutable --check-cache".to_string())
    );
    Ok(())
}

#[test]
fn test_yarn_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/yarn-custom-node-version");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs-14_x"),
            Pkg::new("yarn").set_override("nodejs", "nodejs-14_x")
        ]
    );

    Ok(())
}

#[test]
fn pnpm() -> Result<()> {
    let plan = simple_gen_plan("./examples/pnpm");
    assert_eq!(plan.build.unwrap().cmd, Some("pnpm run build".to_string()));
    assert_eq!(plan.start.unwrap().cmd, Some("pnpm run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_pnpm_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/pnpm-custom-node-version");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs-14_x"),
            Pkg::new("nodePackages.pnpm").set_override("nodejs", "nodejs-14_x")
        ]
    );

    Ok(())
}

#[test]
fn test_go() -> Result<()> {
    let plan = simple_gen_plan("./examples/go");
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("go build -o out main.go".to_string())
    );
    assert_eq!(plan.start.clone().unwrap().cmd, Some("./out".to_string()));
    assert!(plan.start.unwrap().run_image.is_some());

    Ok(())
}

#[test]
fn test_go_cgo_enabled() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/go",
        vec!["CGO_ENABLED=1"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("go build -o out main.go".to_string())
    );
    assert_eq!(plan.start.clone().unwrap().cmd, Some("./out".to_string()));
    assert!(plan.start.unwrap().run_image.is_none());

    Ok(())
}

#[test]
fn test_go_mod() -> Result<()> {
    let plan = simple_gen_plan("./examples/go-mod");
    assert_eq!(plan.build.unwrap().cmd, Some("go build -o out".to_string()));
    assert_eq!(plan.start.unwrap().cmd, Some("./out".to_string()));

    Ok(())
}

#[test]
fn test_go_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/go-custom-version");
    assert_eq!(plan.setup.unwrap().pkgs, vec![Pkg::new("go_1_18")]);

    Ok(())
}

#[test]
fn test_deno() -> Result<()> {
    let plan = simple_gen_plan("./examples/deno");
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("deno cache src/index.ts".to_string())
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("deno run --allow-all src/index.ts".to_string())
    );

    Ok(())
}

#[test]
fn test_procfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/procfile");
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_custom_pkgs() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/hello",
        Vec::new(),
        &GeneratePlanOptions {
            custom_start_cmd: Some("./start.sh".to_string()),
            custom_pkgs: vec![Pkg::new("cowsay")],
            ..Default::default()
        },
    )?;
    assert_eq!(plan.setup.unwrap().pkgs, vec![Pkg::new("cowsay")]);
    assert_eq!(plan.start.unwrap().cmd, Some("./start.sh".to_string()));

    Ok(())
}

#[test]
fn test_pin_archive() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/hello",
        Vec::new(),
        &GeneratePlanOptions {
            pin_pkgs: true,
            ..Default::default()
        },
    )?;
    assert!(plan.setup.unwrap().archive.is_some());

    Ok(())
}

#[test]
fn test_custom_rust_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/rust-custom-version");
    assert!(plan
        .build
        .unwrap()
        .cmd
        .unwrap()
        .contains("cargo build --release"));
    assert_eq!(
        plan.setup
            .unwrap()
            .pkgs
            .iter()
            .filter(|p| p.name.contains("1.56.0"))
            .count(),
        1
    );

    Ok(())
}

#[test]
fn test_rust_rocket() -> Result<()> {
    let plan = simple_gen_plan("./examples/rust-rocket");
    assert!(plan
        .build
        .unwrap()
        .cmd
        .unwrap()
        .contains("cargo build --release"));
    assert!(plan.start.clone().unwrap().cmd.is_some());
    assert_eq!(
        plan.start.clone().unwrap().cmd.unwrap(),
        "./rocket".to_string()
    );
    assert!(plan.start.unwrap().run_image.is_some());

    Ok(())
}

#[test]
fn test_rust_rocket_no_musl() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/rust-rocket",
        vec!["NIXPACKS_NO_MUSL=1"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("cargo build --release".to_string())
    );
    assert!(plan
        .start
        .clone()
        .unwrap()
        .cmd
        .unwrap()
        .contains("./target/release/rocket"));
    assert!(plan.start.unwrap().run_image.is_none());

    Ok(())
}

#[test]
pub fn test_python() -> Result<()> {
    let plan = simple_gen_plan("./examples/python");
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.install.unwrap().cmd,
        Some("python -m venv /opt/venv && . /opt/venv/bin/activate && pip install -r requirements.txt".to_string())
    );
    assert_eq!(plan.start.unwrap().cmd, Some("python main.py".to_string()));

    Ok(())
}

#[test]
pub fn test_python_poetry() -> Result<()> {
    let plan = simple_gen_plan("./examples/python-poetry");
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.install.unwrap().cmd,
        Some("python -m venv /opt/venv && . /opt/venv/bin/activate && pip install poetry==$NIXPACKS_POETRY_VERSION && poetry install --no-dev --no-interaction --no-ansi".to_string())
    );
    assert_eq!(plan.start.unwrap().cmd, Some("python main.py".to_string()));

    Ok(())
}

#[test]
fn test_node_main_file() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-main-file");
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("node src/index.js".to_string())
    );

    Ok(())
}

#[test]
pub fn test_python_setuptools() -> Result<()> {
    let plan = simple_gen_plan("./examples/python-setuptools");
    assert_eq!(plan.build.unwrap().cmd, None);

    if let Some(install_cmd) = plan.install.unwrap().cmd {
        assert!(install_cmd.contains("setuptools"));
    } else {
        return Err(anyhow::anyhow!("no install command"));
    }

    if let Some(start_cmd) = plan.start.unwrap().cmd {
        assert!(start_cmd.contains("python -m"));
    } else {
        return Err(anyhow::anyhow!("no start command"));
    }

    Ok(())
}

#[test]
fn test_node_main_file_doesnt_exist() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-main-file-not-exist");
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_haskell_stack() -> Result<()> {
    let plan = simple_gen_plan("./examples/haskell-stack");
    assert_eq!(plan.build.unwrap().cmd, Some("stack build".to_string()));
    assert!(plan.start.unwrap().cmd.unwrap().contains("stack exec"));
    assert!(plan.install.unwrap().cmd.unwrap().contains("stack setup"));
    Ok(())
}

#[test]
fn test_crystal() -> Result<()> {
    let plan = simple_gen_plan("./examples/crystal");
    assert_eq!(
        plan.install.unwrap().cmd,
        Some("shards install".to_string())
    );
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("shards build --release".to_string())
    );
    assert_eq!(plan.start.unwrap().cmd, Some("./bin/crystal".to_string()));
    Ok(())
}

#[test]
fn test_overriding_environment_variables() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/variables",
        vec!["NODE_ENV=test"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.variables.unwrap().get("NODE_ENV"),
        Some(&"test".to_string())
    );

    Ok(())
}

#[test]
fn test_config_from_environment_variables() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/hello",
        vec![
            "NIXPACKS_PKGS=cowsay ripgrep",
            "NIXPACKS_BUILD_CMD=build",
            "NIXPACKS_START_CMD=start",
            "NIXPACKS_RUN_IMAGE=alpine",
        ],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(plan.build.unwrap().cmd, Some("build".to_string()));
    assert_eq!(plan.start.clone().unwrap().cmd, Some("start".to_string()));
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![Pkg::new("cowsay"), Pkg::new("ripgrep")]
    );
    assert_eq!(plan.start.unwrap().run_image, Some("alpine".to_string()));

    Ok(())
}

#[test]
fn test_staticfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/staticfile");
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("mkdir /etc/nginx/ /var/log/nginx/ /var/cache/nginx/".to_string())
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("nginx -c /assets/nginx.conf".to_string())
    );
    Ok(())
}
