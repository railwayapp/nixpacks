use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;

const DEFAULT_JDK_PKG_NAME: &'static &str = &"jdk8";
pub struct ClojureProvider {}

impl Provider for ClojureProvider {
    fn name(&self) -> &str {
        "clojure"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("project.clj"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![
            Pkg::new("leiningen"),
            Pkg::new("jdk8"),
        ])))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new("lein uberjar".to_string())))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(
            "java $JAVA_OPTS -jar target/uberjar/*standalone.jar".to_string(),
        )))
    }
}

impl ClojureProvider {
    fn get_custom_version(app: &App, env: &Environment) -> Result<String> {
        // Fetch version from configs
        let mut custom_version = env.get_config_variable("JDK_VERSION");

        // If not from configs, get it from the .jdk-version file
        if custom_version.is_none() && app.includes_file(".jdk-version") {
            custom_version = Some(app.read_file(".jdk-version")?);
        }

        match custom_version {
            Some(v) => Ok(v),
            None => Ok(DEFAULT_JDK_PKG_NAME.to_string()),
        }
    }

    fn parse_custom_version(custom_version: String) -> Result<String> {
        // Regex for reading JDK versions (e.g. 8 or 11 or latest)
        let jdk_regex = Regex::new(r"(^[0-9][0-9]?$)|(^latest$)")?;

        // Capture matches
        let matches = jdk_regex.captures(custom_version.as_str().trim());

        // If no matches, just use default
        if matches.is_none() {
            return Ok(DEFAULT_JDK_PKG_NAME.to_string());
        }

        let matches = matches.unwrap();
        let matched_value = if matches.get(0).is_some() {
            matches.get(0)
        } else {
            matches.get(1)
        };

        let value = match matched_value {
            Some(m) => m.as_str(),
            None => "_",
        };

        Ok(value.to_string())
    }

    pub fn get_nix_jdk_package(app: &App, env: &Environment) -> Result<Pkg> {
        let custom_version = ClojureProvider::get_custom_version(app, env)?;
        let parsed_version = ClojureProvider::parse_custom_version(custom_version)?;

        let pkg_name = match parsed_version.as_str() {
            "latest" => "jdk",
            "11" => "jdk11",
            _ => DEFAULT_JDK_PKG_NAME, // 8 or any other value
        };

        Ok(Pkg::new(pkg_name))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nixpacks::{app::App, environment::Environment, nix::pkg::Pkg};
    use std::collections::HashMap;

    #[test]
    fn test_no_version() -> Result<()> {
        assert_eq!(
            ClojureProvider::get_nix_jdk_package(
                &App::new("./examples/clojure")?,
                &Environment::default()
            )?,
            Pkg::new(DEFAULT_JDK_PKG_NAME)
        );

        Ok(())
    }

    #[test]
    fn test_custom_version() -> Result<()> {
        assert_eq!(
            ClojureProvider::get_nix_jdk_package(
                &App::new("./examples/clojure-jdk11")?,
                &Environment::default()
            )?,
            Pkg::new("jdk11")
        );

        Ok(())
    }

    #[test]
    fn test_custom_latest_version() -> Result<()> {
        assert_eq!(
            ClojureProvider::get_nix_jdk_package(
                &App::new("./examples/clojure-jdk-latest")?,
                &Environment::default()
            )?,
            Pkg::new("jdk")
        );

        Ok(())
    }

    #[test]
    fn test_latest_version_from_environment_variable() -> Result<()> {
        assert_eq!(
            ClojureProvider::get_nix_jdk_package(
                &App::new("./examples/clojure-jdk-latest")?,
                &Environment::new(HashMap::from([(
                    "NIXPACKS_JDK_VERSION".to_string(),
                    "latest".to_string()
                )]))
            )?,
            Pkg::new("jdk")
        );

        Ok(())
    }

    #[test]
    fn test_version_from_environment_variable() -> Result<()> {
        assert_eq!(
            ClojureProvider::get_nix_jdk_package(
                &App::new("./examples/clojure")?,
                &Environment::new(HashMap::from([(
                    "NIXPACKS_JDK_VERSION".to_string(),
                    "11".to_string()
                )]))
            )?,
            Pkg::new("jdk11")
        );

        Ok(())
    }
}
