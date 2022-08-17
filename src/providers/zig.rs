use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::legacy_phase::{
        LegacyBuildPhase, LegacyInstallPhase, LegacySetupPhase, LegacyStartPhase,
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

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<LegacySetupPhase>> {
        let mut pkgs = vec![Pkg::new("zig")];
        if app.includes_file("gyro.zzz") {
            pkgs.push(Pkg::new("wget"));
        }
        Ok(Some(LegacySetupPhase::new(pkgs)))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<LegacyInstallPhase>> {
        let mut phase = LegacyInstallPhase::default();
        if app.includes_file(".gitmodules") {
            phase.add_cmd("git submodule update --init".to_string());
        }
        if app.includes_file("gyro.zzz") {
            let gyro_exe_path = format!("/gyro/gyro-{}-linux-{}/bin/gyro", GYRO_VERSION, ARCH);
            phase.add_cmd(format!(
                "mkdir /gyro && (wget -O- {} | tar -C /gyro -xzf -)",
                ZigProvider::get_gyro_download_url()
            ));
            phase.add_cmd(format!("chmod +x {}", gyro_exe_path));
            phase.add_cmd(format!("{} fetch", gyro_exe_path));
        }
        Ok(Some(phase))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<LegacyBuildPhase>> {
        Ok(Some(LegacyBuildPhase::new(
            "zig build -Drelease-safe=true".to_string(),
        )))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<LegacyStartPhase>> {
        Ok(Some(LegacyStartPhase::new(format!(
            "./zig-out/bin/{}",
            app.source
                .file_name()
                .map(OsStr::to_str)
                .map_or("*", Option::unwrap)
        ))))
    }
}

impl ZigProvider {
    pub fn get_gyro_download_url() -> String {
        let gyro_supported_archs: Vec<&str> = vec!["x86_64", "aarch64", "i386"];
        if gyro_supported_archs.contains(&ARCH) {
            format!(
                "https://github.com/mattnite/gyro/releases/download/{}/gyro-{}-linux-{}.tar.gz",
                GYRO_VERSION, GYRO_VERSION, ARCH
            )
        } else {
            panic!("Gyro is not supported on your architecture ({}).", ARCH)
        }
    }
}
