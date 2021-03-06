use std::collections::BTreeMap;

use serde::Deserialize;

use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

use super::Provider;
use std::env::consts::ARCH;

const STACK_CACHE_DIR: &'static &str = &"/root/.stack";
const STACK_WORK_CACHE_DIR: &'static &str = &".stack-work";

pub struct HaskellStackProvider {}

impl Provider for HaskellStackProvider {
    fn name(&self) -> &str {
        "haskell_stack"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.yaml") && app.has_match("**/*.hs"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let mut setup_phase = SetupPhase::new(vec![Pkg::new("stack")]);
        setup_phase.add_apt_pkgs(vec![
            "libgmp-dev".to_string(),
            "gcc".to_string(),
            "binutils".to_string(),
            "make".to_string(),
            "zlib1g-dev".to_string(),
        ]);
        if ARCH == "aarch64" {
            setup_phase.add_apt_pkgs(vec![
                "libnuma1".to_string(),
                "libnuma-dev".to_string(),
                "libtinfo-dev".to_string(),
                "libtinfo5".to_string(),
                "libc6-dev".to_string(),
                "libtinfo6".to_string(),
                "llvm-11".to_string(),
                "clang".to_string(),
                "ninja-build".to_string(),
                "zlib1g-dev".to_string(),
            ])
        }

        Ok(Some(setup_phase))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("stack setup".to_string());
        install_phase.add_cache_directory(STACK_CACHE_DIR.to_string());

        Ok(Some(install_phase))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let mut build_phase = BuildPhase::new("stack install".to_string());
        build_phase.add_cache_directory(STACK_CACHE_DIR.to_string());
        build_phase.add_cache_directory(STACK_WORK_CACHE_DIR.to_string());

        Ok(Some(build_phase))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let package: HaskellStackPackageYaml = app.read_yaml("package.yaml")?;
        let exe_names: Vec<String> = package.executables.keys().cloned().collect();

        let name = exe_names
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable name"))?;

        Ok(Some(StartPhase::new(format!("/root/.local/bin/{name}",))))
    }
}

#[derive(Deserialize)]
struct HaskellStackPackageYaml {
    pub executables: BTreeMap<String, HaskellStackExecutableDefinition>,
}

#[derive(Deserialize)]
struct HaskellStackExecutableDefinition {}
