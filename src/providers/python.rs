use crate::{
    chain,
    nixpacks::{
        app::App,
        asdf::parse_tool_versions_content,
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

const DEFAULT_PYTHON_PKG_NAME: &str = "python3";
const POETRY_VERSION: &str = "1.3.1";
const PDM_VERSION: &str = "2.13.3";
const UV_VERSION: &str = "0.4.30";

const VENV_LOCATION: &str = "/opt/venv";
const UV_CACHE_DIR: &str = "/root/.cache/uv";
const PIP_CACHE_DIR: &str = "/root/.cache/pip";
const PDM_CACHE_DIR: &str = "/root/.cache/pdm";
const DEFAULT_POETRY_PYTHON_PKG_NAME: &str = "python3";

const PYTHON_NIXPKGS_ARCHIVE: &str = "bc8f8d1be58e8c8383e683a06e1e1e57893fff87";
const LEGACY_PYTHON_NIXPKGS_ARCHIVE: &str = "5148520bfab61f99fd25fb9ff7bfbb50dad3c9db";

pub struct PythonProvider {}

impl Provider for PythonProvider {
    fn name(&self) -> &'static str {
        "python"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        let has_python = app.includes_file("main.py")
            || app.includes_file("requirements.txt")
            || app.includes_file("pyproject.toml")
            || app.includes_file("Pipfile");
        Ok(has_python)
    }

    fn metadata(&self, app: &App, env: &Environment) -> Result<ProviderMetadata> {
        let is_django = PythonProvider::is_django(app, env)?;
        let is_using_postgres = PythonProvider::is_using_postgres(app, env)?;
        let is_poetry = app.includes_file("poetry.lock");
        let is_pdm = app.includes_file("pdm.lock");

        Ok(ProviderMetadata::from(vec![
            (is_django, "django"),
            (is_using_postgres, "postgres"),
            (is_poetry, "poetry"),
            (is_pdm, "pdm"),
        ]))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut plan = BuildPlan::default();

        let setup = self.setup(app, env)?.unwrap_or_default();
        plan.add_phase(setup);

        let install = self.install(app, env)?.unwrap_or_default();
        plan.add_phase(install);

        if let Some(start) = self.start(app, env)? {
            plan.set_start_phase(start);
        }

        plan.add_variables(PythonProvider::default_python_environment_variables());

        if app.includes_file("poetry.lock") {
            let mut version = POETRY_VERSION.to_string();

            if app.includes_file(".tool-versions") {
                let file_content = &app.read_file(".tool-versions")?;

                if let Some(poetry_version) =
                    PythonProvider::parse_tool_versions_poetry_version(file_content)?
                {
                    version = poetry_version;
                }
            }

            plan.add_variables(EnvironmentVariables::from([(
                "NIXPACKS_POETRY_VERSION".to_string(),
                version,
            )]));
        }

        if app.includes_file("pdm.lock") {
            plan.add_variables(EnvironmentVariables::from([(
                "NIXPACKS_PDM_VERSION".to_string(),
                PDM_VERSION.to_string(),
            )]));
        }

        // uv version is not, as of 0.4.30, specified in the lock file or pyproject.toml
        if app.includes_file("uv.lock") {
            let mut version = UV_VERSION.to_string();

            if app.includes_file(".tool-versions") {
                let file_content = &app.read_file(".tool-versions")?;

                if let Some(uv_version) =
                    PythonProvider::parse_tool_versions_uv_version(file_content)?
                {
                    version = uv_version;
                }
            }

            plan.add_variables(EnvironmentVariables::from([
                ("NIXPACKS_UV_VERSION".to_string(), version),
                (
                    "UV_PROJECT_ENVIRONMENT".to_string(),
                    VENV_LOCATION.to_string(),
                ),
            ]));
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
        let mut setup = Phase::setup(None);

        let mut pkgs: Vec<Pkg> = vec![];
        let (python_base_package, nix_archive) = PythonProvider::get_nix_python_package(app, env)?;

        pkgs.append(&mut vec![python_base_package]);

        if PythonProvider::is_using_postgres(app, env)? {
            pkgs.append(&mut vec![Pkg::new("postgresql_16.dev")]);
        }

        if PythonProvider::is_django(app, env)? && PythonProvider::is_using_mysql(app, env)? {
            // We need the MySQL client library and its headers to build the mysqlclient python module needed by Django
            pkgs.append(&mut vec![Pkg::new("libmysqlclient.dev")]);
        }

        if app.includes_file("Pipfile") {
            pkgs.append(&mut vec![Pkg::new("pipenv")]);
        }

        setup.add_nix_pkgs(&pkgs);
        setup.set_nix_archive(nix_archive);

        if PythonProvider::uses_dep(app, "cairo")? {
            setup.add_pkgs_libs(vec!["cairo".to_string()]);
        }

        // Many Python packages need some C headers to be available
        // stdenv.cc.cc.lib -> https://discourse.nixos.org/t/nixos-with-poetry-installed-pandas-libstdc-so-6-cannot-open-shared-object-file/8442/3
        setup.add_pkgs_libs(vec!["zlib".to_string(), "stdenv.cc.cc.lib".to_string()]);
        setup.add_nix_pkgs(&[Pkg::new("gcc")]);

        Ok(Some(setup))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<Phase>> {
        let create_env = format!("python -m venv --copies {VENV_LOCATION}");
        let activate_env = format!(". {VENV_LOCATION}/bin/activate");

        if app.includes_file("requirements.txt") {
            let mut install_phase = Phase::install(Some(format!(
                "{create_env} && {activate_env} && pip install -r requirements.txt"
            )));

            install_phase.add_path(format!("{VENV_LOCATION}/bin"));
            install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

            return Ok(Some(install_phase));
        } else if app.includes_file("pyproject.toml") {
            if app.includes_file("poetry.lock") {
                let install_poetry = "pip install poetry==$NIXPACKS_POETRY_VERSION".to_string();
                let mut install_phase = Phase::install(Some(format!(
                    "{create_env} && {activate_env} && {install_poetry} && poetry install --no-dev --no-interaction --no-ansi"
                )));

                install_phase.add_path(format!("{VENV_LOCATION}/bin"));

                install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

                return Ok(Some(install_phase));
            } else if app.includes_file("pdm.lock") {
                let install_pdm = "pip install pdm==$NIXPACKS_PDM_VERSION".to_string();
                let mut install_phase = Phase::install(Some(format!(
                    "{create_env} && {activate_env} && {install_pdm} && pdm install --prod"
                )));

                install_phase.add_path(format!("{VENV_LOCATION}/bin"));

                install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());
                install_phase.add_cache_directory(PDM_CACHE_DIR.to_string());

                return Ok(Some(install_phase));
            } else if app.includes_file("uv.lock") {
                let install_uv = "pip install uv==$NIXPACKS_UV_VERSION".to_string();

                // Here's how we get UV to play well with the pre-existing non-standard venv location:
                //
                // 1. Create a venv which allows us to use pip. pip is not installed globally with nixpkgs py
                // 2. Install uv via pip
                // 3. UV_PROJECT_ENVIRONMENT is specified elsewhere so `uv sync` installs packages into the same venv

                let mut install_phase = Phase::install(Some(format!(
                    "{create_env} && {activate_env} && {install_uv} && uv sync --no-dev --frozen"
                )));

                install_phase.add_path(format!("{VENV_LOCATION}/bin"));
                install_phase.add_cache_directory(UV_CACHE_DIR.to_string());

                return Ok(Some(install_phase));
            }

            let mut install_phase = Phase::install(Some(format!(
                "{create_env} && {activate_env} && pip install --upgrade build setuptools && pip install ."
            )));

            install_phase.add_file_dependency("pyproject.toml".to_string());
            install_phase.add_path(format!("{VENV_LOCATION}/bin"));

            install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

            return Ok(Some(install_phase));
        } else if app.includes_file("Pipfile") {
            // By default Pipenv creates an environment directory in some random location (for example `/root/.local/share/virtualenvs/app-4PlAip0Q`).
            // `PIPENV_VENV_IN_PROJECT` tells it that there is an already activated `venv` environment, So Pipenv will use the same directory instead of creating new one (in our case it's `/app/.venv`)

            let cmd = if app.includes_file("Pipfile.lock") {
                "PIPENV_VENV_IN_PROJECT=1 pipenv install --deploy"
            } else {
                "PIPENV_VENV_IN_PROJECT=1 pipenv install --skip-lock"
            };

            let cmd = format!("{create_env} && {activate_env} && {cmd}");
            let mut install_phase = Phase::install(Some(cmd));

            install_phase.add_path(format!("{VENV_LOCATION}/bin"));
            install_phase.add_cache_directory(PIP_CACHE_DIR.to_string());

            return Ok(Some(install_phase));
        }

        Ok(Some(Phase::install(None)))
    }

    fn start(&self, app: &App, env: &Environment) -> Result<Option<StartPhase>> {
        if PythonProvider::is_django(app, env)? {
            let app_name = PythonProvider::get_django_app_name(app, env)?;

            return Ok(Some(StartPhase::new(format!(
                "python manage.py migrate && gunicorn {app_name}"
            ))));
        }

        // the python package is extracted from pyproject.toml, but this can often not be the desired entrypoint
        // for this reason we prefer main.py to the module heuristic used in the pyproject.toml logic
        if app.includes_file("main.py") {
            return Ok(Some(StartPhase::new("python main.py".to_string())));
        }

        if app.includes_file("pyproject.toml") {
            if let OkResult(meta) = PythonProvider::parse_pyproject(app) {
                if let Some(entry_point) = meta.entry_point {
                    return Ok(Some(StartPhase::new(match entry_point {
                        EntryPoint::Command(cmd) => cmd,
                        EntryPoint::Module(module) => format!("python -m {module}"),
                    })));
                }
            }
        }

        Ok(None)
    }

    fn is_django(app: &App, _env: &Environment) -> Result<bool> {
        let has_manage = app.includes_file("manage.py");
        let imports_django = PythonProvider::uses_dep(app, "django")?;

        Ok(has_manage && imports_django)
    }

    fn is_using_postgres(app: &App, _env: &Environment) -> Result<bool> {
        // Check for the engine database type in settings.py
        let re = Regex::new(r"django.db.backends.postgresql").unwrap();

        let uses_pg =
            app.find_match(&re, "/**/*.py")? || PythonProvider::uses_dep(app, "psycopg2")?;
        Ok(uses_pg)
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
        bail!("Failed to find your WSGI_APPLICATION django setting. Add this to continue.")
    }

    fn parse_pipfile_python_version(file_content: &str) -> Result<Option<String>> {
        let matches = Regex::new("(python_version|python_full_version) = ['|\"]([0-9|.]*)")?
            .captures(file_content);

        Ok(matches
            .filter(|m| m.len() > 2)
            .map(|m| m.get(2).unwrap().as_str().to_string()))
    }

    fn parse_tool_versions_python_version(file_content: &str) -> Result<Option<String>> {
        let asdf_versions = parse_tool_versions_content(file_content);

        // the python version can only specify a major.minor version right now, and not a patch version
        // however, in asdf a patch version is specified, so we need to strip it
        Ok(asdf_versions.get("python").map(|s| {
            let parts: Vec<&str> = s.split('.').collect();

            // We expect there to be 3 or 2 parts (x.y.z) however, only x.y can be parsed.
            // So we accept strip x.y.z -> x.y and warn that all other formats are invalid
            if parts.len() != 3 && parts.len() != 2 {
                eprintln!("Could not find a python version string in the format x.y.z or x.y from .tool-versions. Found {}. Skipping", parts.join("."));
            }

            format!("{}.{}", parts[0], parts[1])
        }))
    }

    fn parse_tool_versions_poetry_version(file_content: &str) -> Result<Option<String>> {
        let asdf_versions = parse_tool_versions_content(file_content);
        Ok(asdf_versions.get("poetry").cloned())
    }

    fn parse_tool_versions_uv_version(file_content: &str) -> Result<Option<String>> {
        let asdf_versions = parse_tool_versions_content(file_content);
        Ok(asdf_versions.get("uv").cloned())
    }

    fn default_python_environment_variables() -> EnvironmentVariables {
        let python_variables = vec![
            ("PYTHONFAULTHANDLER", "1"),
            ("PYTHONUNBUFFERED", "1"),
            ("PYTHONHASHSEED", "random"),
            ("PYTHONDONTWRITEBYTECODE", "1"),
            // TODO I think this would eliminate the need to include the cache version
            ("PIP_NO_CACHE_DIR", "1"),
            ("PIP_DISABLE_PIP_VERSION_CHECK", "1"),
            ("PIP_DEFAULT_TIMEOUT", "100"),
        ];

        let mut env_vars = EnvironmentVariables::new();

        for (key, value) in python_variables {
            env_vars.insert(key.to_owned(), value.to_owned());
        }

        env_vars
    }

    fn get_nix_python_package(app: &App, env: &Environment) -> Result<(Pkg, String)> {
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
        } else if app.includes_file("Pipfile") {
            let file_content = &app.read_file("Pipfile")?;
            custom_version = PythonProvider::parse_pipfile_python_version(file_content)?;
        } else if app.includes_file(".tool-versions") {
            let file_content = &app.read_file(".tool-versions")?;
            custom_version = PythonProvider::parse_tool_versions_python_version(file_content)?;
        }

        // If it's still none, return default
        if custom_version.is_none() {
            if app.includes_file("poetry.lock") {
                return Ok((
                    Pkg::new(DEFAULT_POETRY_PYTHON_PKG_NAME),
                    PYTHON_NIXPKGS_ARCHIVE.into(),
                ));
            }
            return Ok((
                Pkg::new(DEFAULT_PYTHON_PKG_NAME),
                PYTHON_NIXPKGS_ARCHIVE.into(),
            ));
        }
        let custom_version = custom_version.unwrap();

        // Regex for reading Python versions (e.g. 3.8.0 or 3.8 or 3)
        let python_regex =
            Regex::new(r#"^(?:[\sa-zA-Z-"']*)(\d*)(?:\.*)(\d*)(?:\.*\d*)(?:["']?)$"#)?;

        // Capture matches
        let matches = python_regex.captures(custom_version.as_str().trim());

        // If no matches, just use default
        if matches.is_none() {
            if app.includes_file("poetry.lock") {
                return Ok((
                    Pkg::new(DEFAULT_POETRY_PYTHON_PKG_NAME),
                    PYTHON_NIXPKGS_ARCHIVE.into(),
                ));
            }
            return Ok((
                Pkg::new(DEFAULT_PYTHON_PKG_NAME),
                PYTHON_NIXPKGS_ARCHIVE.into(),
            ));
        }

        let matches = matches.unwrap();
        let python_version = (as_default(matches.get(1)), as_default(matches.get(2)));

        // Match major and minor versions
        match python_version {
            ("3", "13") => Ok((Pkg::new("python313"), PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "12") => Ok((Pkg::new("python312"), PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "11") => Ok((Pkg::new("python311"), PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "10") => Ok((Pkg::new("python310"), PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "9") => Ok((Pkg::new("python39"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "8") => Ok((Pkg::new("python38"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())),
            ("3", "7") => Ok((Pkg::new("python37"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())),
            ("2", "7" | "_") => Ok((Pkg::new("python27"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())),
            _ => {
                if app.includes_file("poetry.lock") {
                    return Ok((
                        Pkg::new(DEFAULT_POETRY_PYTHON_PKG_NAME),
                        PYTHON_NIXPKGS_ARCHIVE.into(),
                    ));
                }
                Ok((
                    Pkg::new(DEFAULT_PYTHON_PKG_NAME),
                    PYTHON_NIXPKGS_ARCHIVE.into(),
                ))
            }
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
                |pkgs| pkgs.first().cloned()
            );
            (
                |proj| proj.py_modules,
                |mods| mods.first().cloned()
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

    fn uses_dep(app: &App, dep: &str) -> Result<bool> {
        let is_used = ["requirements.txt", "pyproject.toml", "Pipfile"]
            .iter()
            .any(|f| {
                app.includes_file(f)
                    && app
                        .read_file(f)
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(dep)
            });

        Ok(is_used)
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
            (
                Pkg::new(DEFAULT_PYTHON_PKG_NAME),
                PYTHON_NIXPKGS_ARCHIVE.into()
            )
        );

        Ok(())
    }

    #[test]
    fn test_pipfile_python_version() -> Result<()> {
        let file_content = "\npython_version = '3.12'\n";
        let custom_version = PythonProvider::parse_pipfile_python_version(file_content)?.unwrap();

        assert_eq!(custom_version, "3.12");

        Ok(())
    }

    #[test]
    fn test_pipfile_python_full_version() -> Result<()> {
        let file_content = "\npython_full_version = '3.12.0'\n";
        let custom_version = PythonProvider::parse_pipfile_python_version(file_content)?.unwrap();

        assert_eq!(custom_version, "3.12.0");

        Ok(())
    }

    #[test]
    fn test_custom_version() -> Result<()> {
        assert_eq!(
            PythonProvider::get_nix_python_package(
                &App::new("./examples/python-2")?,
                &Environment::default()
            )?,
            (Pkg::new("python27"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())
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
            (Pkg::new("python27"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())
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
            (Pkg::new("python27"), LEGACY_PYTHON_NIXPKGS_ARCHIVE.into())
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

    #[test]
    fn test_postgres_detection() -> Result<()> {
        assert!(PythonProvider::is_using_postgres(
            &App::new("./examples/python-postgres",)?,
            &Environment::new(BTreeMap::new())
        )
        .unwrap());
        assert!(PythonProvider::is_using_postgres(
            &App::new("./examples/python-django",)?,
            &Environment::new(BTreeMap::new())
        )
        .unwrap());
        assert!(!PythonProvider::is_using_postgres(
            &App::new("./examples/python-django-mysql",)?,
            &Environment::new(BTreeMap::new())
        )
        .unwrap());
        Ok(())
    }

    #[test]
    fn test_django_mysql_detection() -> Result<()> {
        assert!(!PythonProvider::is_using_mysql(
            &App::new("./examples/python-django",)?,
            &Environment::new(BTreeMap::new())
        )
        .unwrap());
        assert!(PythonProvider::is_using_mysql(
            &App::new("./examples/python-django-mysql",)?,
            &Environment::new(BTreeMap::new())
        )
        .unwrap());
        Ok(())
    }
}
