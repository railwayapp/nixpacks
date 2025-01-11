use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    app::App,
    environment::Environment,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};

use super::Provider;

#[derive(Serialize, Deserialize, Debug)]
struct GleamPackageSpec {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GleamManifest {
    pub packages: Vec<GleamPackageSpec>,
}

impl GleamManifest {
    fn get_package_version(&self, package: &str) -> Option<String> {
        Some(
            self.packages
                .iter()
                .find(|pkg| pkg.name == *package)?
                .version
                .clone(),
        )
    }
}

pub struct GleamProvider;

impl Provider for GleamProvider {
    fn name(&self) -> &'static str {
        "gleam"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.has_match("gleam.toml") && app.has_match("manifest.toml"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = self.get_setup(app, env);
        let install = self.get_install(app, env)?;
        let build = self.get_build(app, env);
        let start = self.get_start(app, env);

        let mut plan = BuildPlan::new(&[setup, install, build], Some(start));

        plan.add_static_assets(static_asset_list! {
            "get-gleam.sh" => include_str!("get-gleam.sh")
        });

        Ok(Some(plan))
    }
}

impl GleamProvider {
    fn get_setup(&self, _app: &App, _env: &Environment) -> Phase {
        // erlang and gleam are required, elixir just in case
        let pkgs = vec![
            "wget".into(),
            "erlang".into(),
            "elixir".into(),
            "rebar3".into(),
        ];

        Phase::setup(Some(pkgs))
    }

    fn get_install(&self, app: &App, _env: &Environment) -> Result<Phase> {
        let manifest: GleamManifest = app.read_toml("manifest.toml")?;

        let gleam_version = manifest.get_package_version("gleam_stdlib"); // steal the gleam version from the stdlib version

        let mut phase = Phase::install(Some(format!(
            "sh {} {}",
            app.asset_path("get-gleam.sh"),
            gleam_version.unwrap_or_else(|| "main".into())
        )));
        phase.only_include_files = Some(vec!["gleam.toml".into(), "manifest.toml".into()]);
        phase.add_cmd("gleam deps download");

        Ok(phase)
    }

    fn get_build(&self, _app: &App, _env: &Environment) -> Phase {
        Phase::build(Some("gleam export erlang-shipment".into()))
    }

    fn get_start(&self, _app: &App, _env: &Environment) -> StartPhase {
        let mut phase = StartPhase::new("./build/erlang-shipment/entrypoint.sh run");
        phase.only_include_files = Some(vec!["build/erlang-shipment".into()]);
        phase
    }
}
