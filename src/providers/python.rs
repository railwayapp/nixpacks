use crate::{
    chain,
    nixpacks::{
        app::App,
        environment::{Environment, EnvironmentVariables},
        plan::{
            phase::{Phase, StartPhase},
            BuildPlan,
        },
    },
    Pkg,
};
use anyhow::{bail, Context, Ok, Result};
use regex::{Match, Regex};
use serde::Deserialize;
use std::result::Result::Ok as OkResult;
use std::{collections::HashMap, fs};

use super::{Provider, ProviderMetadata};

const DEFAULT_PYTHON_PKG_NAME: &str = "python38";
const POETRY_VERSION: &str = "1.1.13";
const PIP_CACHE_DIR: &str = "/root/.cache/pip";

pub struct PythonProvider {}

impl Provider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        let has_python = app.includes_file("main.py")
            || app.includes_file("requirements.txt")
            || app.includes_file("pyproject.toml");
        Ok(has_python)
    }

    fn metadata(&self, app: &App, env: &Environment) -> Result<ProviderMetadata> {
        let is_django = PythonProvider::is_django(app, env)?;
        let is_using_postgres = PythonProvider::is_using_postgres(app, env)?;
        let is_poetry = app.includes_file("poetry.lock");

        Ok(ProviderMetadata::from(vec![
            (is_django, "django"),
            (is_using_postgres, "postgres"),
            (is_poetry, "poetry"),
        ]))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        if let Some(setup) = self.setup(app, env)? {
            plan.add_phase(setup);
        }
        if let Some(install) = self.install(app, env)? {
            plan.add_phase(install);
        }
        if let Some(start) = self.start(app, env)? {
            plan.set_start_phase(start);
        }

        if app.includes_file("poetry.lock") {
            plan.add_variables(EnvironmentVariables::from([(
                "NIXPACKS_POETRY_VERSION".to_string(),
                POETRY_VERSION.to_string(),
            )]));
        }

        Ok(Some(plan))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct PyProject {
    pub project: Option<ProjectDecl>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ProjectDecl {
    pub name: Option<String>,
    pub packages: Option<Vec<String>>,
    pub py_modules: Option<Vec<String>>,
    pub entry_points: Option<HashMap<String, String>>,
}

#[allow(dead_code)]
struct ProjectMeta {
    pub project_name: Option<String>,
    pub module_name: Option<String>,
    pub entry_point: Option<EntryPoint>,
}

#[allow(dead_code)]
enum EntryPoint {
    Command(String),
    Module(String),
}

impl PythonProvider {
    fn setup(&self, app: &App, env: &Environment) -> Result<Option<Phase>> {
        let mut pkgs: Vec<Pkg> = vec![];
        let python_base_package = PythonProvider::get_nix_python_package(app, env)?;

        pkgs.append(&mut vec![python_base_package]);

        if PythonProvider::is_django(app, env)? && PythonProvider::is_using_postgres(app, env)? {
            // Django with Postgres requires postgresql and gcc on top of the original python packages
            pkgs.append(&mut vec![Pkg::new("postgresql")]);
        }

        if PythonProvider::is_django(app, env)? && PythonProvider::is_using_mysql(app, env)? {
            // We need the MySQL client library and its headers to build the mysqlclient python module needed by Django
            pkgs.append(&mut vec![Pkg::new("libmysqlclient.dev")]);
        }

        let mut setup_phase = Phase::setup(Some(pkgs));

        // Many Python packages need some C headers to be available
        // stdenv.cc.cc.lib -> https://discourse.nixos.org/t/nixos-with-poetry-installed-pandas-libstdc-so-6-cannot-open-shared-object-file/8442/3
        setup_phase.add_pkgs_libs(vec!["zlib".to_string(), "stdenv.cc.cc.lib".to_string()]);
        setup_phase.add_nix_pkgs(vec![Pkg::new("gcc")]);

        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<Phase>> {
        let env_loc = "/opt/venv";
        let create_env = format!("python -m venv {}", env_loc);
        let activate_env = format!(". {}/bin/activate", env_loc);

        if app.includes_file("requirements.txt") {
            let mut install_phase = Phase::install(Some(format!(
                "{} && {} && pip install -r requirements.txt",
                create_env, activate_env
            )));

            install_phase.add_path(format!("{}/bin", env_loc));
            install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

            return Ok(Some(install_phase));
        } else if app.includes_file("pyproject.toml") {
            if app.includes_file("poetry.lock") {
                let install_poetry = "pip install poetry==$NIXPACKS_POETRY_VERSION".to_string();
                let mut install_phase = Phase::install(Some(format!(
                    "{} && {} && {} && poetry install --no-dev --no-interaction --no-ansi",
                    create_env, activate_env, install_poetry
                )));

                install_phase.add_path(format!("{}/bin", env_loc));

                install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

                return Ok(Some(install_phase));
            }
            let mut install_phase = Phase::install(Some(format!(
                "{} && {} && pip install --upgrade build setuptools && pip install .",
                create_env, activate_env
            )));

            install_phase.add_file_dependency("pyproject.toml".to_string());
            install_phase.add_path(format!("{}/bin", env_loc));

            install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

            return Ok(Some(install_phase));
        }

        Ok(None)
    }

    fn start(&self, app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        if PythonProvider::is_django(app, env)? {
            let app_name = PythonProvider::get_django_app_name(app, env)?;

            return Ok(Some(StartPhase::new(format!(
                "python manage.py migrate && gunicorn {}",
                app_name
            ))));
        }

        if app.includes_file("pyproject.toml") {
            if let OkResult(meta) = PythonProvider::parse_pyproject(app) {
                if let Some(entry_point) = meta.entry_point {
                    return Ok(Some(StartPhase::new(match entry_point {
                        EntryPoint::Command(cmd) => cmd,
                        EntryPoint::Module(module) => format!("python -m {}", module),
                    })));
                }
            }
        }
        // falls through
        if app.includes_file("main.py") {
            return Ok(Some(StartPhase::new("python main.py".to_string())));
        }

        Ok(None)
    }

    fn is_django(app: &App, _env: &Environment) -> Result<bool> {
        let has_manage = app.includes_file("manage.py");
        let imports_django = vec!["requirements.txt", "pyproject.toml"].iter().any(|f| {
            app.read_file(f)
                .unwrap_or_default()
                .to_lowercase()
                .contains("django")
        });
        Ok(has_manage && imports_django)
    }

    fn is_using_postgres(app: &App, _env: &Environment) -> Result<bool> {
        // Check for the engine database type in settings.py
        let re = Regex::new(r"django.db.backends.postgresql").unwrap();

        app.find_match(&re, "/**/*.py")
    }

    fn is_using_mysql(app: &App, _env: &Environment) -> Result<bool> {
        // django_psdb_engine is a PlanetScale specific variant of django.db.backends.mysql
        let re = Regex::new(r"django\.db\.backends\.mysql|django_psdb_engine").unwrap();
        app.find_match(&re, "/**/*.py")
    }

    fn get_django_app_name(app: &App, _env: &Environment) -> Result<String> {
        // Look for the settings.py file
        let paths = app.find_files("/**/*.py").unwrap();

        // Generate regex to find the application name
        let re = Regex::new(r#"WSGI_APPLICATION = ["|'](.*).application["|']"#).unwrap();

        // Search all settings.py matches
        for path in paths {
            let path_buf = fs::canonicalize(path)?;

            if let Some(p) = path_buf.to_str() {
                let f = app.read_file(p)?;
                if let Some(value) = re.captures(f.as_str()) {
                    // Get the first and only match
                    // e.g "mysite.wsgi"
                    return Ok(value.get(1).unwrap().as_str().into());
                }
            }
        }
        bail!("Failed to find django application name!")
    }

    fn get_nix_python_package(app: &App, env: &Environment) -> Result<Pkg> {
        // Fetch python versions into tuples with defaults
        fn as_default(v: Option<Match>) -> &str {
            match v {
                Some(m) => m.as_str(),
                None => "_",
            }
        }

        // Fetch version from configs
        let mut custom_version = env.get_config_variable("PYTHON_VERSION");

        // If not from configs, get it from the .python-version file
        if custom_version.is_none() && app.includes_file(".python-version") {
            custom_version = Some(app.read_file(".python-version")?);
        } else if app.includes_file("runtime.txt") {
            custom_version = Some(app.read_file("runtime.txt")?);
        }

        // If it's still none, return default
        if custom_version.is_none() {
            return Ok(Pkg::new(DEFAULT_PYTHON_PKG_NAME));
        }
        let custom_version = custom_version.unwrap();

        // Regex for reading Python versions (e.g. 3.8.0 or 3.8 or 3)
        let python_regex =
            Regex::new(r#"^(?:[\sa-zA-Z-"']*)(\d*)(?:\.*)(\d*)(?:\.*\d*)(?:["']?)$"#)?;

        // Capture matches
        let matches = python_regex.captures(custom_version.as_str().trim());

        // If no matches, just use default
        if matches.is_none() {
            return Ok(Pkg::new(DEFAULT_PYTHON_PKG_NAME));
        }
        let matches = matches.unwrap();
        let python_version = (as_default(matches.get(1)), as_default(matches.get(2)));

        // Match major and minor versions
        match python_version {
            ("3", "11") => Ok(Pkg::new("python311")),
            ("3", "10") => Ok(Pkg::new("python310")),
            ("3", "9") => Ok(Pkg::new("python39")),
            ("3", "8") => Ok(Pkg::new("python38")),
            ("3", "7") => Ok(Pkg::new("python37")),
            ("2", "7" | "_") => Ok(Pkg::new("python27")),
            _ => Ok(Pkg::new(DEFAULT_PYTHON_PKG_NAME)),
        }
    }

    fn read_pyproject(app: &App) -> Result<Option<PyProject>> {
        if app.includes_file("pyproject.toml") {
            return Ok(Some(
                app.read_toml("pyproject.toml")
                    .context("Reading pyproject.toml")?,
            ));
        }
        Ok(None)
    }

    fn parse_project(project: &PyProject) -> ProjectMeta {
        let project_name = project
            .project
            .as_ref()
            .and_then(|proj| proj.name.as_ref())
            .cloned();

        let module_name = chain!(project.project.clone() =>
            (
                |proj| proj.packages,
                |pkgs| pkgs.get(0).cloned()
            );
            (
                |proj| proj.py_modules,
                |mods| mods.get(0).cloned()
            );
            (
                |_| project_name.clone()
            )
        );

        let entry_point = module_name.clone().map(EntryPoint::Module);

        ProjectMeta {
            project_name,
            module_name,
            entry_point,
        }
    }

    fn parse_pyproject(app: &App) -> Result<ProjectMeta> {
        Ok(PythonProvider::parse_project(
            &(PythonProvider::read_pyproject(app)?
                .ok_or_else(|| anyhow::anyhow!("failed to load pyproject.toml"))?),
        ))
    }

    #[allow(dead_code)]
    fn uses_dep(app: &App, dep: &str) -> Result<bool> {
        let requirements_usage = app.includes_file("requirements.txt")
            && app
                .read_file("requirements.txt")?
                .to_lowercase()
                .contains(dep);

        let pyproject_usage = app.includes_file("pyproject.toml")
            && app
                .read_file("pyproject.toml")?
                .to_lowercase()
                .contains(dep);

        Ok(requirements_usage || pyproject_usage)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nixpacks::{app::App, environment::Environment, nix::pkg::Pkg};
    use std::collections::BTreeMap;

    #[test]
    fn test_no_version() -> Result<()> {
        assert_eq!(
            PythonProvider::get_nix_python_package(
                &App::new("./examples/python")?,
                &Environment::default()
            )?,
            Pkg::new(DEFAULT_PYTHON_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_custom_version() -> Result<()> {
        assert_eq!(
            PythonProvider::get_nix_python_package(
                &App::new("./examples/python-2")?,
                &Environment::default()
            )?,
            Pkg::new("python27")
        );

        Ok(())
    }

    #[test]
    fn test_custom_version_runtime_txt() -> Result<()> {
        assert_eq!(
            PythonProvider::get_nix_python_package(
                &App::new("./examples/python-2-runtime")?,
                &Environment::default()
            )?,
            Pkg::new("python27")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_environment_variable() -> Result<()> {
        assert_eq!(
            PythonProvider::get_nix_python_package(
                &App::new("./examples/python")?,
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_PYTHON_VERSION".to_string(),
                    "2.7".to_string()
                )]))
            )?,
            Pkg::new("python27")
        );

        Ok(())
    }

    #[test]
    fn test_numpy_detection() -> Result<()> {
        assert!(!PythonProvider::uses_dep(
            &App::new("./examples/python",)?,
            "numpy"
        )?,);
        assert!(PythonProvider::uses_dep(
            &App::new("./examples/python-numpy",)?,
            "numpy"
        )?,);
        Ok(())
    }
}
