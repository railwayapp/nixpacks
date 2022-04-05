use anyhow::Result;
use nixpacks::{gen_plan, providers::Pkg};

#[test]
fn test_node() -> Result<()> {
    let plan = gen_plan("./examples/node", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(plan.start_cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_npm() -> Result<()> {
    let plan = gen_plan("./examples/npm", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, Some("npm run build".to_string()));
    assert_eq!(plan.start_cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_node_no_scripts() -> Result<()> {
    let plan = gen_plan("./examples/node-no-scripts", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(plan.start_cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_yarn() -> Result<()> {
    let plan = gen_plan("./examples/yarn", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, Some("yarn build".to_string()));
    assert_eq!(plan.start_cmd, Some("yarn start".to_string()));

    Ok(())
}

#[test]
fn test_go() -> Result<()> {
    let plan = gen_plan("./examples/go", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(plan.start_cmd, Some("go run main.go".to_string()));

    Ok(())
}

#[test]
fn test_deno() -> Result<()> {
    let plan = gen_plan("./examples/deno", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(
        plan.start_cmd,
        Some("deno run --allow-all src/index.ts".to_string())
    );

    Ok(())
}

#[test]
fn test_procfile() -> Result<()> {
    let plan = gen_plan("./examples/procfile", Vec::new(), None, None, false)?;
    assert_eq!(plan.start_cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_custom_pkgs() -> Result<()> {
    let plan = gen_plan(
        "./examples/hello",
        vec!["cowsay"],
        None,
        Some("./start.sh".to_string()),
        false,
    )?;
    assert_eq!(plan.pkgs, vec![Pkg::new("cowsay")]);
    assert_eq!(plan.start_cmd, Some("./start.sh".to_string()));

    Ok(())
}

#[test]
fn test_pin_archive() -> Result<()> {
    let plan = gen_plan("./examples/hello", Vec::new(), None, None, true)?;
    assert!(plan.nixpkgs_archive.is_some());

    Ok(())
}

#[test]
fn test_rust_rocket() -> Result<()> {
    let plan = gen_plan("./examples/rust-rocket", Vec::new(), None, None, true)?;
    assert_eq!(plan.build_cmd, Some("cargo build --release".to_string()));
    assert!(plan.start_cmd.is_some());

    if let Some(start_cmd) = plan.start_cmd {
        assert!(start_cmd.contains("./target/release/rocket"));
    }

    Ok(())
}

#[test]
fn test_node_main_file() -> Result<()> {
    let plan = gen_plan("./examples/node-main-file", Vec::new(), None, None, false)?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(plan.start_cmd, Some("node src/index.js".to_string()));

    Ok(())
}

#[test]
fn test_node_main_file_doesnt_exist() -> Result<()> {
    let plan = gen_plan(
        "./examples/node-main-file-not-exist",
        Vec::new(),
        None,
        None,
        false,
    )?;
    assert_eq!(plan.build_cmd, None);
    assert_eq!(plan.start_cmd, Some("node index.js".to_string()));

    Ok(())
}
