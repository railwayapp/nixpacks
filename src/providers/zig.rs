use crate::nixpacks::{
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};

use super::Provider;

pub struct ZigProvider;

impl Provider for ZigProvider {
    fn setup(
        &self,
        _app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("zig")])))
    }

    fn install(
        &self,
        app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::InstallPhase>> {
        Ok(if app.includes_file(".gitmodules") {
            Some(InstallPhase::new(format!(
                "bash {}",
                app.asset_path("zig-install.sh")
            )))
        } else {
            None
        })
    }

    fn build(
        &self,
        _app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::BuildPhase>> {
        Ok(Some(BuildPhase::new(
            "zig build -Drelease-safe=true".to_string(),
        )))
    }

    fn start(
        &self,
        app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::StartPhase>> {
        Ok(Some(StartPhase::new(format!(
            "./zig-out/bin/{}",
            app.source
                .file_name()
                .expect("Failed to determine project name")
                .to_str()
                .unwrap()
        ))))
    }

    fn static_assets(
        &self,
        _app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::app::StaticAssets>> {
        Ok(Some(static_asset_list!(
            "zig-install.sh" => include_str!("zig/install-phase.sh")
        )))
    }

    fn name(&self) -> &str {
        "zig"
    }

    fn detect(
        &self,
        app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<bool> {
        Ok(app.has_match("*.zig") || app.has_match("**/*.zig"))
    }
}
