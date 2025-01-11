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

pub struct ScalaProvider {}

const DEFAULT_JDK_VERSION: u32 = 17;

/**
 * Scala provider currently supports sbt.
 * - The sbt project requires sbt-native-packager, a popular packaging
 *   tool used by the community to package apps. Setting executableScriptName and
 *   enabling the JavaAppPackaging plugin are required. Please check examples/scala-sbt
 *   for an example.
 *
 * TODO: Add support for scala-cli and mill
 */
impl Provider for ScalaProvider {
    fn name(&self) -> &'static str {
        "scala"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("build.sbt"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        if self.is_using_sbt(app) {
            let jdk_version: u32 = self.get_jdk_version(env);

            let pkgs = self.get_sbt_dep_pkgs(jdk_version);
            let setup = Phase::setup(Some(pkgs));

            let mut build = Phase::build(None);
            let sbt_exe = self.get_sbt_exe();

            build.add_cmd(format!("{sbt_exe} stage"));
            build.add_cache_directory("/root/.sbt");
            build.add_cache_directory("/root/.ivy2/cache");
            build.add_cache_directory("/root/.cache/coursier");
            build.depends_on_phase("setup");

            let start_phase = self.get_start_cmd(app).map(StartPhase::new).map(|phase| {
                let mut updated_phase = phase;
                updated_phase.run_in_image(self.get_jdk_run_image(jdk_version).to_string());
                updated_phase.add_file_dependency("./target/universal");
                updated_phase
            });

            let plan = BuildPlan::new(&vec![setup, build], start_phase);
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }
}

impl ScalaProvider {
    fn get_sbt_exe(&self) -> String {
        "sbt".to_string()
    }

    fn get_start_cmd(&self, app: &App) -> Option<String> {
        if self.is_using_sbt(app) {
            Some("./target/universal/stage/bin/main".to_string())
        } else {
            None
        }
    }

    fn get_jdk_pkg_name(&self, jdk_version: u32) -> &str {
        match jdk_version {
            21 => "jdk21",
            20 => "jdk20",
            19 => "jdk",
            11 => "jdk11",
            8 => "jdk8",

            // Using 17 as default because its the latest LTS
            _ => "jdk17",
        }
    }

    fn get_jdk_run_image(&self, jdk_version: u32) -> &str {
        match jdk_version {
            21 => "eclipse-temurin:21.0.1_12-jre-jammy",
            20 => "eclipse-temurin:20.0.2_9-jre-jammy",
            19 => "eclipse-temurin:19.0.2_7-jre-jammy",
            11 => "eclipse-temurin:11.0.21_9-jre-jammy",
            8 => "eclipse-temurin:8u392-b08-jre-jammy",

            // Using 17 as default because its the latest LTS
            _ => "eclipse-temurin:17.0.9_9-jre-jammy",
        }
    }

    fn is_using_sbt(&self, app: &App) -> bool {
        app.includes_file("build.sbt")
    }

    pub fn get_sbt_dep_pkgs(&self, jdk_version: u32) -> Vec<Pkg> {
        let pkgs = vec![self.get_sbt_pkg(jdk_version)];
        pkgs
    }

    pub fn get_jdk_version(&self, env: &Environment) -> u32 {
        env.get_config_variable("JDK_VERSION")
            .map_or(DEFAULT_JDK_VERSION, |env_string| {
                env_string.parse::<u32>().unwrap()
            })
    }

    fn get_sbt_pkg(&self, jdk_version: u32) -> Pkg {
        Pkg::new("sbt").set_override("jre", self.get_jdk_pkg_name(jdk_version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_jdk_pkg_name() {
        let scala = ScalaProvider {};

        // defaults to Java 17
        assert_eq!(
            "jdk17",
            scala
                .get_jdk_pkg_name(scala.get_jdk_version(&Environment::from_envs(vec![]).unwrap(),))
        );

        // Supports Java 20
        assert_eq!(
            "jdk20",
            scala.get_jdk_pkg_name(scala.get_jdk_version(
                &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=20"]).unwrap(),
            ))
        );

        // Supports Java 21
        assert_eq!(
            "jdk21",
            scala.get_jdk_pkg_name(scala.get_jdk_version(
                &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=21"]).unwrap(),
            ))
        );
    }

    #[test]
    fn test_sbt_package() {
        let scala = ScalaProvider {};

        assert!(scala.is_using_sbt(&App::new("examples/scala-sbt").unwrap()));
        assert!(!scala.is_using_sbt(&App::new("examples/node").unwrap()));
        assert_eq!(
            Pkg::new("sbt").set_override("jre", "jdk8"),
            scala.get_sbt_pkg(
                scala.get_jdk_version(
                    &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=8"]).unwrap(),
                )
            )
        );
    }
}
