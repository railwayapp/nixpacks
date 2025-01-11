use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Result};

pub struct FSharpProvider {}

pub const ARTIFACT_DIR: &str = "out";

impl Provider for FSharpProvider {
    fn name(&self) -> &'static str {
        "f#"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(!app.find_files("*.fsproj")?.is_empty())
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![Pkg::new("dotnet-sdk")]));
        let install = Phase::install(Some("dotnet restore".to_string()));
        let build = Phase::build(Some(format!(
            "dotnet publish --no-restore -c Release -o {ARTIFACT_DIR}"
        )));

        let fsproj = &app.find_files("*.fsproj")?[0].with_extension("");
        let project_name = fsproj
            .file_name()
            .context("Invalid file_name")?
            .to_str()
            .context("Invalid project_name")?;
        let start = StartPhase::new(format!("./{ARTIFACT_DIR}/{project_name}"));

        let mut plan = BuildPlan::new(&vec![setup, install, build], Some(start));
        plan.add_variables(EnvironmentVariables::from([
            (
                "ASPNETCORE_ENVIRONMENT".to_string(),
                "Production".to_string(),
            ),
            (
                "ASPNETCORE_URLS".to_string(),
                "http://0.0.0.0:3000".to_string(),
            ),
            (
                "DOTNET_ROOT".to_string(),
                "/nix/var/nix/profiles/default/".to_string(),
            ),
        ]));

        Ok(Some(plan))
    }
}
