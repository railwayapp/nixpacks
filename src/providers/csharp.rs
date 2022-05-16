use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{Context, Result};

pub struct CSharpProvider {}

pub const ARTIFACT_DIR: &'static &str = &"out";

impl Provider for CSharpProvider {
    fn name(&self) -> &str {
        "csharp"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(!app.find_files("*.csproj")?.is_empty())
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![Pkg::new("dotnet-sdk")])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        Ok(Some(InstallPhase::new("dotnet restore".to_string())))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new(format!(
            "dotnet publish --no-restore -c Release -o {}",
            ARTIFACT_DIR
        ))))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let csproj = &app.find_files("*.csproj")?[0].with_extension("");
        let project_name = csproj
            .file_name()
            .unwrap()
            .to_str()
            .context("Invalid project_name")?;
        Ok(Some(StartPhase::new(format!(
            "./{}/{}",
            ARTIFACT_DIR,
            project_name
        ))))
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        let env_vars = EnvironmentVariables::from([
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
        ]);
        Ok(Some(env_vars))
    }
}
