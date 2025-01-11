use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::Result;
use regex::Regex;

const DEFAULT_JDK_PKG_NAME: &str = "jdk8";

pub struct ClojureProvider {}

impl Provider for ClojureProvider {
    fn name(&self) -> &'static str {
        "clojure"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(self.is_using_lein(app) || self.is_using_tools_build(app))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = Phase::setup(Some(vec![
            if self.is_using_tools_build(app) {
                Pkg::new("clojure")
            } else {
                Pkg::new("leiningen")
            },
            ClojureProvider::get_nix_jdk_package(app, env)?,
        ]));

        let build_cmd = if self.is_using_tools_build(app) {
            "clojure -T:build uber"
        } else if self.has_lein_ring_plugin(app) {
            "lein ring uberjar"
        } else {
            "lein uberjar"
        };

        /*
          Based on project config, uberjar can be created under ./target/uberjar or ./target. This ensures we always find the jar file in the root target folder.
        */
        let move_file_cmd = r#"if [ -d /app/target/default+uberjar ]; then find /app/target/default+uberjar -name "*.jar" -exec mv -t /app/target/ {} +; fi"#;

        let mut build = Phase::build(Some(format!("{build_cmd}; {move_file_cmd}")));
        build.depends_on_phase("setup");

        /*
          Looks for a file ending with standalone.jar or .jar but not *-SNAPSHOT.jar and starts the app
        */
        let start = StartPhase::new(
            r#"JAR_FILE=$(find /app/target -name "*-standalone.jar" -o -name "*.jar" ! -name "*-SNAPSHOT.jar" | head -n 1) && bash -c "java $JAVA_OPTS -jar $JAR_FILE""#,
        );

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}

impl ClojureProvider {
    fn has_lein_ring_plugin(&self, app: &App) -> bool {
        self.is_using_lein(app)
            && app
                .read_file("project.clj")
                .unwrap_or_default()
                .to_lowercase()
                .contains("[lein-ring ")
    }

    fn is_using_lein(&self, app: &App) -> bool {
        app.includes_file("project.clj")
    }

    fn is_using_tools_build(&self, app: &App) -> bool {
        app.includes_file("build.clj")
    }

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

    fn parse_custom_version(custom_version: &str) -> Result<String> {
        // Regex for reading JDK versions (e.g. 8 or 11 or latest)
        let jdk_regex = Regex::new(r"^([0-9][0-9]?|latest)$")?;

        // Capture matches
        let matches = jdk_regex.captures(custom_version.trim());

        // If no matches, just use default
        if matches.is_none() {
            return Ok(DEFAULT_JDK_PKG_NAME.to_string());
        }

        let matches = matches.unwrap();
        let matched_value = matches.get(0);

        let value = match matched_value {
            Some(m) => m.as_str(),
            None => "_",
        };

        Ok(value.to_string())
    }

    pub fn get_nix_jdk_package(app: &App, env: &Environment) -> Result<Pkg> {
        let custom_version = ClojureProvider::get_custom_version(app, env)?;
        let parsed_version = ClojureProvider::parse_custom_version(&custom_version)?;

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
    use std::collections::BTreeMap;

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
                &Environment::new(BTreeMap::from([(
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
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_JDK_VERSION".to_string(),
                    "11".to_string()
                )]))
            )?,
            Pkg::new("jdk11")
        );

        Ok(())
    }
}
