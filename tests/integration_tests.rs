use anyhow::Result;
use nixpacks::{gen_plan, nixpacks::nix::Pkg};

#[test]
fn test_node() -> Result<()> {
    let plan = gen_plan("./examples/node", Vec::new(), None, None, Vec::new(), false)?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_npm() -> Result<()> {
    let plan = gen_plan("./examples/npm", Vec::new(), None, None, Vec::new(), false)?;
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
    let plan = gen_plan(
        "./examples/node-no-scripts",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_node_custom_version() -> Result<()> {
    let plan = gen_plan(
        "./examples/node-custom-version",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(
        plan.setup.unwrap().nix.pkgs,
        vec![Pkg::new("pkgs.stdenv"), Pkg::new("nodejs-12_x")]
    );

    Ok(())
}

#[test]
fn test_yarn() -> Result<()> {
    let plan = gen_plan("./examples/yarn", Vec::new(), None, None, Vec::new(), false)?;
    assert_eq!(plan.build.unwrap().cmd, Some("yarn build".to_string()));
    assert_eq!(plan.start.unwrap().cmd, Some("yarn start".to_string()));
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
fn test_yarn_custom_version() -> Result<()> {
    let plan = gen_plan(
        "./examples/yarn-custom-node-version",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(
        plan.setup.unwrap().nix.pkgs,
        vec![
            Pkg::new("pkgs.stdenv"),
            Pkg::new("pkgs.yarn").set_override("nodejs", "nodejs-14_x")
        ]
    );

    Ok(())
}

#[test]
fn test_go() -> Result<()> {
    let plan = gen_plan("./examples/go", Vec::new(), None, None, Vec::new(), false)?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("go run main.go".to_string()));

    Ok(())
}

#[test]
fn test_deno() -> Result<()> {
    let plan = gen_plan("./examples/deno", Vec::new(), None, None, Vec::new(), false)?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("deno run --allow-all src/index.ts".to_string())
    );

    Ok(())
}

#[test]
fn test_procfile() -> Result<()> {
    let plan = gen_plan(
        "./examples/procfile",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_custom_pkgs() -> Result<()> {
    let plan = gen_plan(
        "./examples/hello",
        vec!["cowsay"],
        None,
        Some("./start.sh".to_string()),
        Vec::new(),
        false,
    )?;
    assert_eq!(plan.setup.unwrap().nix.pkgs, vec![Pkg::new("cowsay")]);
    assert_eq!(plan.start.unwrap().cmd, Some("./start.sh".to_string()));

    Ok(())
}

#[test]
fn test_pin_archive() -> Result<()> {
    let plan = gen_plan("./examples/hello", Vec::new(), None, None, Vec::new(), true)?;
    assert!(plan.setup.unwrap().nix.archive.is_some());

    Ok(())
}

#[test]
fn test_custom_rust_version() -> Result<()> {
    let plan = gen_plan(
        "./examples/rust-custom-version",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("cargo build --release".to_string())
    );
    assert_eq!(
        plan.setup
            .unwrap()
            .nix
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
    let plan = gen_plan(
        "./examples/rust-rocket",
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
    )?;
    assert_eq!(
        plan.build.unwrap().cmd,
        Some("cargo build --release".to_string())
    );
    assert!(plan.start.clone().unwrap().cmd.is_some());

    if let Some(start_cmd) = plan.start.unwrap().cmd {
        assert!(start_cmd.contains("./target/release/rocket"));
    }

    Ok(())
}

#[test]
pub fn test_python() -> Result<()> {
    let plan = gen_plan(
        "./examples/python",
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
    )?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.install.unwrap().cmd,
        Some("python -m ensurepip && python -m pip install -r requirements.txt".to_string())
    );
    assert_eq!(plan.start.unwrap().cmd, Some("python main.py".to_string()));

    Ok(())
}

#[test]
fn test_node_main_file() -> Result<()> {
    let plan = gen_plan(
        "./examples/node-main-file",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("node src/index.js".to_string())
    );

    Ok(())
}

#[test]
pub fn test_python_setuptools() -> Result<()> {
    let plan = gen_plan(
        "./examples/python-setuptools",
        Vec::new(),
        None,
        None,
        Vec::new(),
        true,
    )?;
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
    let plan = gen_plan(
        "./examples/node-main-file-not-exist",
        Vec::new(),
        None,
        None,
        Vec::new(),
        false,
    )?;
    assert_eq!(plan.build.unwrap().cmd, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_overriding_environment_variables() -> Result<()> {
    let plan = gen_plan(
        "./examples/variables",
        Vec::new(),
        None,
        None,
        vec!["NODE_ENV=test"],
        false,
    )?;
    assert_eq!(
        plan.variables.unwrap().get("NODE_ENV"),
        Some(&"test".to_string())
    );

    Ok(())
}
