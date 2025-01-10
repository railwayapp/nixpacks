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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CSharpSdk {
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CSharpGlobalJson {
    pub sdk: Option<CSharpSdk>,
}

pub struct CSharpProvider {}

pub const ARTIFACT_DIR: &str = "out";

impl Provider for CSharpProvider {
    fn name(&self) -> &'static str {
        "c#"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(!app.find_files("*.csproj")?.is_empty())
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let sdk = CSharpProvider::get_sdk_version(app, env);
        let setup = Phase::setup(Some(vec![Pkg::new(sdk?.as_str())]));
        let install = Phase::install(Some("dotnet restore".to_string()));
        let build = Phase::build(Some(format!(
            "dotnet publish --no-restore -c Release -o {ARTIFACT_DIR}"
        )));

        let csproj = &app.find_files("*.csproj")?[0].with_extension("");
        let project_name = csproj
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

impl CSharpProvider {
    fn get_sdk_version(app: &App, env: &Environment) -> Result<String> {
        // Check if a version is specified in global.json
        let global_json = if app.includes_file("global.json") {
            let global_json: CSharpGlobalJson = app.read_json("global.json")?;
            global_json.sdk.and_then(|sdk| sdk.version)
        } else {
            None
        };
        // Use environment variable then global_json then default to 6
        let version_string = env
            .get_config_variable("CSHARP_SDK_VERSION")
            .or(global_json)
            .or_else(|| Some(String::from("6")));
        let version_number: u8 = version_string
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .parse()
            .unwrap_or(6); // split by '.', get first item, attempt to parse to u8, if not default to 6
        match version_number {
            6 => Ok("dotnet-sdk".to_string()),
            _ => Ok(format!("dotnet-sdk_{version_number}")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nixpacks::{app::App, environment::Environment};
    use std::collections::BTreeMap;

    #[test]
    fn test_no_version() -> Result<()> {
        let expected_sdk_name = "dotnet-sdk";
        assert_eq!(
            CSharpProvider::get_sdk_version(
                &App::new("./examples/csharp-cli")?,
                &Environment::default()
            )?,
            expected_sdk_name
        );

        Ok(())
    }

    #[test]
    fn test_global_json() -> Result<()> {
        let expected_sdk_name = "dotnet-sdk_7";
        assert_eq!(
            CSharpProvider::get_sdk_version(
                &App::new("./examples/csharp-api")?,
                &Environment::default()
            )?,
            expected_sdk_name
        );

        Ok(())
    }

    #[test]
    fn test_version_from_environment_variable() -> Result<()> {
        let expected_sdk_name = "dotnet-sdk";
        assert_eq!(
            CSharpProvider::get_sdk_version(
                &App::new("./examples/csharp-cli")?,
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_CSHARP_SDK_VERSION".to_string(),
                    "6.0.0".to_string()
                )]))
            )?,
            expected_sdk_name
        );

        Ok(())
    }
}
