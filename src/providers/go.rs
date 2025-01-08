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
const AVAILABLE_GO_VERSIONS: &[(&str, &str, &str)] = &[
    (
        "1.18",
        "go_1_18",
        "5148520bfab61f99fd25fb9ff7bfbb50dad3c9db",
    ),
    (
        "1.19",
        "go_1_19",
        "5148520bfab61f99fd25fb9ff7bfbb50dad3c9db",
    ),
    (
        "1.20",
        "go_1_20",
        "1f13eabcd6f5b00fe9de9575ac52c66a0e887ce6",
    ),
    (
        "1.21",
        "go_1_21",
        "1f13eabcd6f5b00fe9de9575ac52c66a0e887ce6",
    ),
    ("1.22", "go", "e89cf1c932006531f454de7d652163a9a5c86668"),
    (
        "1.23",
        "go_1_23",
        "05bbf675397d5366259409139039af8077d695ce",
    ),
];
const DEFAULT_GO_PKG_NAME: &str = "go";
const DEFAULT_ARCHIVE: &str = "e89cf1c932006531f454de7d652163a9a5c86668";

const GO_BUILD_CACHE_DIR: &str = "/root/.cache/go-build";

impl Provider for GolangProvider {
    fn name(&self) -> &str {
        "go"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("main.go") || app.includes_file("go.mod"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let go_mod = self.read_go_mod_if_exists(app)?;
        let (nix_pkg, archive) = GolangProvider::get_nix_golang_pkg(go_mod.as_ref())?;

        let mut setup = Phase::setup(Some(vec![Pkg::new(&nix_pkg)]));
        setup.set_nix_archive(archive);

        plan.add_phase(setup);
        let is_go_module = app.includes_file("go.mod");

        if is_go_module {
            let mut install = Phase::install(Some("go mod download".to_string()));
            install.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());
            plan.add_phase(install);
        }

        let has_root_go_files = app.find_files("*.go").ok().map_or(false, |files| {
            files
                .iter()
                .any(|file| file.parent() == Some(app.source.as_path()))
        });

        let build_command = if let Some(name) = env.get_config_variable("GO_BIN") {
            Some(format!("go build -o {BINARY_NAME} ./cmd/{name}"))
        } else if is_go_module && has_root_go_files {
            Some(format!("go build -o {BINARY_NAME}"))
        } else if app.includes_directory("cmd") {
            // Try to find a command in the cmd directory
            app.find_directories("cmd/*")
                .ok()
                .and_then(|dirs| {
                    dirs.into_iter()
                        .find(|path| path.parent().map_or(false, |p| p.ends_with("cmd")))
                })
                .and_then(|path| {
                    path.file_name()
                        .and_then(|os_str| os_str.to_str())
                        .map(|name| format!("go build -o {BINARY_NAME} ./cmd/{name}"))
                })
        } else if is_go_module {
            Some(format!("go build -o {BINARY_NAME}"))
        } else if app.includes_file("main.go") {
            Some(format!("go build -o {BINARY_NAME} main.go"))
        } else {
            None
        };

        let mut build = Phase::build(build_command);
        build.add_cache_directory(GO_BUILD_CACHE_DIR.to_string());
        build.depends_on_phase("setup");
        plan.add_phase(build);

        let has_go_files = app.has_match("**/*.go");

        if has_go_files {
            let mut start = StartPhase::new(format!("./{BINARY_NAME}"));
            let cgo = env.get_variable("CGO_ENABLED").unwrap_or("0");

            // Only run in a new image if CGO_ENABLED=0 (default)
            if cgo != "1" {
                start.run_in_slim_image();
            }
            plan.set_start_phase(start);
        }

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

    pub fn get_nix_golang_pkg(go_mod_contents: Option<&String>) -> Result<(String, String)> {
        if go_mod_contents.is_some() {
            let mut lines = go_mod_contents.as_ref().unwrap().lines();
            let go_version_line = lines.find(|line| line.trim().starts_with("go"));

            if let Some(go_version_line) = go_version_line {
                let go_version = go_version_line.split_whitespace().nth(1).unwrap();

                let nix_pkg = version_number_to_pkg(go_version)
                    .unwrap_or_else(|| DEFAULT_GO_PKG_NAME.to_string());
                let nix_archive = version_number_to_archive(go_version)
                    .unwrap_or_else(|| DEFAULT_ARCHIVE.to_string());

                return Ok((nix_pkg, nix_archive));
            }
        }

        Ok((DEFAULT_GO_PKG_NAME.to_string(), DEFAULT_ARCHIVE.to_string()))
    }
}

fn version_number_to_pkg(version: &str) -> Option<String> {
    let matched_version = AVAILABLE_GO_VERSIONS.iter().find(|(v, _, _)| v == &version);
    matched_version.map(|(_, pkg, _)| (*pkg).to_string())
}

fn version_number_to_archive(version: &str) -> Option<String> {
    let matched_version = AVAILABLE_GO_VERSIONS.iter().find(|(v, _, _)| v == &version);
    matched_version.map(|(_, _, archive)| (*archive).to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_go_mod() -> Result<()> {
        assert_eq!(
            GolangProvider::get_nix_golang_pkg(None)?.0,
            DEFAULT_GO_PKG_NAME.to_string()
        );

        Ok(())
    }

    #[test]
    fn test_with_go_mod() -> Result<()> {
        let go_mod_contents = r"
            go 1.18
        ";

        assert_eq!(
            GolangProvider::get_nix_golang_pkg(Some(&go_mod_contents.to_string()))?.0,
            "go_1_18".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_fallback_on_invalid_version() -> Result<()> {
        let go_mod_contents = r"
            go 1.8
        ";

        assert_eq!(
            GolangProvider::get_nix_golang_pkg(Some(&go_mod_contents.to_string()))?.0,
            DEFAULT_GO_PKG_NAME.to_string()
        );

        Ok(())
    }
}
