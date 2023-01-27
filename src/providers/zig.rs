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
use std::{env::consts::ARCH, ffi::OsStr};

pub struct ZigProvider;

//TODO: CHANGE THIS WHEN ZIG IS UPDATED OR EVERYTHING WILL BREAK!
const GYRO_VERSION: &str = "0.6.0";

impl Provider for ZigProvider {
    fn name(&self) -> &str {
        "zig"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.has_match("*.zig") || app.has_match("**/*.zig") || app.has_match("gyro.zzz"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut setup = Phase::setup(Some(vec![Pkg::new("zig")]));

        if app.includes_file("gyro.zzz") {
            setup.add_nix_pkgs(&[Pkg::new("wget")]);
        }

        let mut install = Phase::install(None);
        if app.includes_file(".gitmodules") {
            install.add_cmd("git submodule update --init".to_string());
        }
        if app.includes_file("gyro.zzz") {
            let gyro_exe_path = format!("/gyro/gyro-{GYRO_VERSION}-linux-{ARCH}/bin/gyro");
            install.add_cmd(format!(
                "mkdir /gyro && (wget -O- {} | tar -C /gyro -xzf -)",
                ZigProvider::get_gyro_download_url()
            ));
            install.add_cmd(format!("chmod +x {gyro_exe_path}"));
            install.add_cmd(format!("{gyro_exe_path} fetch"));
        }

        let build = Phase::build(Some("zig build -Drelease-safe=true".to_string()));

        let start = StartPhase::new(format!(
            "./zig-out/bin/{}",
            app.source
                .file_name()
                .map(OsStr::to_str)
                .map_or("*", Option::unwrap)
        ));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        Ok(Some(plan))
    }
}

impl ZigProvider {
    pub fn get_gyro_download_url() -> String {
        let gyro_supported_archs: Vec<&str> = vec!["x86_64", "aarch64", "i386"];
        if gyro_supported_archs.contains(&ARCH) {
            format!(
                "https://github.com/mattnite/gyro/releases/download/{GYRO_VERSION}/gyro-{GYRO_VERSION}-linux-{ARCH}.tar.gz"
            )
        } else {
            panic!("Gyro is not supported on your architecture ({ARCH}).")
        }
    }
}
