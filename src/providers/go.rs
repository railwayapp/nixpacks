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
use anyhow::Result;

pub struct GolangProvider {}

const BINARY_NAME: &str = "out";
const AVAILABLE_GO_VERSIONS: &[(&str, &str)] = &[("1.17", "go"), ("1.18", "go_1_18")];
const DEFAULT_GO_PKG_NAME: &str = "go";

const GO_BUILD_CACHE_DIR: &str = "/root/.cache/go-build";

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "golang"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.go") || app.includes_file("go.mod"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let go_mod = self.read_go_mod_if_exists(app)?;
        let nix_pkg = GolangProvider::get_nix_golang_pkg(go_mod.as_ref())?;
        plan.add_phase(Phase::setup(Some(vec![Pkg::new(&nix_pkg)])));

        if app.includes_file("go.mod") {
            let mut install = Phase::install(Some("go mod download".to_string()));
            install.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());
            plan.add_phase(install);
        }

        let mut build = if app.includes_file("go.mod") {
            Phase::build(Some(format!("go build -o {}", BINARY_NAME)))
        } else {
            Phase::build(Some(format!("go build -o {} main.go", BINARY_NAME)))
        };
        build.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());
        plan.add_phase(build);

        let mut start = StartPhase::new(format!("./{}", BINARY_NAME));
        let cgo = env.get_variable("CGO_ENABLED").unwrap_or("0");
        // Only run in a new image if CGO_ENABLED=0 (default)
        if cgo != "1" {
            start.run_in_slim_image();
        }
        plan.set_start_phase(start);

        plan.add_variables(EnvironmentVariables::from([(
            "CGO_ENABLED".to_string(),
            "0".to_string(),
        )]));

        Ok(Some(plan))
    }
}

impl GolangProvider {
    pub fn read_go_mod_if_exists(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("go.mod") {
            Ok(Some(app.read_file("go.mod")?))
        } else {
            Ok(None)
        }
    }

    pub fn get_nix_golang_pkg(go_mod_contents: Option<&String>) -> Result<String> {
        if go_mod_contents.is_some() {
            let mut lines = go_mod_contents.as_ref().unwrap().lines();
            let go_version_line = lines.find(|line| line.trim().starts_with("go"));

            if let Some(go_version_line) = go_version_line {
                let go_version = go_version_line.split_whitespace().nth(1).unwrap();

                if let Some(nix_pkg) = version_number_to_pkg(go_version) {
                    return Ok(nix_pkg);
                }
            }
        }

        Ok(DEFAULT_GO_PKG_NAME.to_string())
    }
}

fn version_number_to_pkg(version: &str) -> Option<String> {
    let matched_version = AVAILABLE_GO_VERSIONS.iter().find(|(v, _)| v == &version);

    matched_version.map(|(_, pkg)| (*pkg).to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_go_mod() -> Result<()> {
        assert_eq!(
            GolangProvider::get_nix_golang_pkg(None)?,
            DEFAULT_GO_PKG_NAME.to_string()
        );

        Ok(())
    }

    #[test]
    fn test_with_go_mod() -> Result<()> {
        let go_mod_contents = r#"
            go 1.18
        "#;

        assert_eq!(
            GolangProvider::get_nix_golang_pkg(Some(&go_mod_contents.to_string()))?,
            "go_1_18".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_fallback_on_invalid_version() -> Result<()> {
        let go_mod_contents = r#"
            go 1.8
        "#;

        assert_eq!(
            GolangProvider::get_nix_golang_pkg(Some(&go_mod_contents.to_string()))?,
            DEFAULT_GO_PKG_NAME.to_string()
        );

        Ok(())
    }
}
