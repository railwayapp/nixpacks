use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env::consts::ARCH;

const STACK_CACHE_DIR: &str = "/root/.stack";
const STACK_WORK_CACHE_DIR: &str = ".stack-work";

pub struct HaskellStackProvider {}

impl Provider for HaskellStackProvider {
    fn name(&self) -> &'static str {
        "haskell"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("package.yaml") && app.has_match("**/*.hs"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut setup = Phase::setup(Some(vec![Pkg::new("stack")]));
        setup.add_apt_pkgs(vec![
            "libgmp-dev".to_string(),
            "gcc".to_string(),
            "binutils".to_string(),
            "make".to_string(),
            "zlib1g-dev".to_string(),
        ]);
        if ARCH == "aarch64" {
            setup.add_apt_pkgs(vec![
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
            ]);
        }

        let mut install = Phase::install(Some("stack setup".to_string()));
        install.add_cache_directory(STACK_CACHE_DIR.to_string());

        let mut build = Phase::build(Some("stack install".to_string()));
        build.add_cache_directory(STACK_CACHE_DIR.to_string());
        build.add_cache_directory(STACK_WORK_CACHE_DIR.to_string());

        let package: HaskellStackPackageYaml = app.read_yaml("package.yaml")?;
        let exe_names: Vec<String> = package.executables.keys().cloned().collect();

        let name = exe_names
            .first()
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable name"))?;

        let start = StartPhase::new(format!("/root/.local/bin/{name}"));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));

        Ok(Some(plan))
    }
}

#[derive(Deserialize)]
#[allow(clippy::zero_sized_map_values)]
struct HaskellStackPackageYaml {
    pub executables: BTreeMap<String, HaskellStackExecutableDefinition>,
}

#[derive(Deserialize)]
struct HaskellStackExecutableDefinition {}
