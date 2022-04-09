use anyhow::Result;

use crate::{
    nixpacks::{app::App, environment::Environment, nix::NixConfig},
    python::pyproject,
    Pkg,
};

use super::Provider;

pub struct PythonProvider {}
impl Provider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn detect(&self, app: &crate::nixpacks::app::App, _env: &Environment) -> anyhow::Result<bool> {
        Ok(app.includes_file("main.py")
            || app.includes_file("requirements.txt")
            || app.includes_file("pyproject.toml"))
    }

    fn install_cmd(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        if app.includes_file("requirements.txt") {
            return Ok(Some(
                "python -m ensurepip && python -m pip install -r requirements.txt".to_string(),
            ));
        } else if app.includes_file("pyproject.toml") {
            return Ok(Some("python -m ensurepip && python -m pip install --upgrade build setuptools && python -m pip install .".to_string()));
        }
        Ok(None)
    }

    fn suggested_build_cmd(&self, _app: &App, _env: &Environment) -> Result<Option<String>> {
        Ok(None)
    }

    fn suggested_start_command(&self, app: &App, _env: &Environment) -> Result<Option<String>> {
        if app.includes_file("pyproject.toml") {
            if let Ok(meta) = pyproject::parse(app) {
                if let Some(entry_point) = meta.entry_point {
                    return match entry_point {
                        pyproject::EntryPoint::Command(cmd) => Ok(Some(cmd)),
                        pyproject::EntryPoint::Module(module) => {
                            Ok(Some(format!("python -m {}", module)))
                        }
                    };
                }
            }
        }
        // falls through
        if app.includes_file("main.py") {
            return Ok(Some("python main.py".to_string()));
        }
        Ok(None)
    }

    fn nix_config(
        &self,
        _app: &App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> Result<crate::nixpacks::nix::NixConfig> {
        Ok(NixConfig::new(vec![Pkg::new("pkgs.python38")]))
    }
}
