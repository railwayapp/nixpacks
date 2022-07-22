use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;

pub struct GolangProvider {}

const BINARY_NAME: &'static &str = &"out";
const AVAILABLE_GO_VERSIONS: &[(&str, &str)] = &[("1.17", "go"), ("1.18", "go_1_18")];
const DEFAULT_GO_PKG_NAME: &'static &str = &"go";

const GO_BUILD_CACHE_DIR: &'static &str = &"/root/.cache/go-build";

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "golang"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.go") || app.includes_file("go.mod"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let go_mod = self.read_go_mod_if_exists(_app)?;
        let nix_pkg = GolangProvider::get_nix_golang_pkg(go_mod)?;

        Ok(Some(SetupPhase::new(vec![Pkg::new(&nix_pkg)])))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        if app.includes_file("go.mod") {
            let mut install_phase = InstallPhase::new("go get".to_string());
            install_phase.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());
            return Ok(Some(install_phase));
        }
        Ok(None)
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let mut build_phase = if app.includes_file("go.mod") {
            BuildPhase::new(format!("go build -o {}", BINARY_NAME))
        } else {
            BuildPhase::new(format!("go build -o {} main.go", BINARY_NAME))
        };

        build_phase.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());

        Ok(Some(build_phase))
    }

    fn start(&self, _app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        let mut start_phase = StartPhase::new(format!("./{}", BINARY_NAME));

        let cgo = env.get_variable("CGO_ENABLED").unwrap_or("0");

        // Only run in a new image if CGO_ENABLED=0 (default)
        if cgo != "1" {
            start_phase.run_in_slim_image();
        }

        Ok(Some(start_phase))
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(EnvironmentVariables::from([(
            "CGO_ENABLED".to_string(),
            "0".to_string(),
        )])))
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

    pub fn get_nix_golang_pkg(go_mod_contents: Option<String>) -> Result<String> {
        if go_mod_contents.is_some() {
            let mut lines = go_mod_contents.as_ref().unwrap().lines();
            let go_version_line = lines.find(|line| line.trim().starts_with("go"));

            if let Some(go_version_line) = go_version_line {
                let go_version = go_version_line.split_whitespace().nth(1).unwrap();

                if let Some(nix_pkg) = version_number_to_pkg(go_version)? {
                    return Ok(nix_pkg);
                }
            }
        }

        Ok(DEFAULT_GO_PKG_NAME.to_string())
    }
}

fn version_number_to_pkg(version: &str) -> Result<Option<String>> {
    let matched_version = AVAILABLE_GO_VERSIONS.iter().find(|(v, _)| v == &version);

    match matched_version {
        Some((_, pkg)) => Ok(Some(pkg.to_string())),
        None => Ok(None),
    }
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
            GolangProvider::get_nix_golang_pkg(Some(go_mod_contents.to_string()))?,
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
            GolangProvider::get_nix_golang_pkg(Some(go_mod_contents.to_string()))?,
            DEFAULT_GO_PKG_NAME.to_string()
        );

        Ok(())
    }
}
