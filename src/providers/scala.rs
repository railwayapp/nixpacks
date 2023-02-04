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
    fn name(&self) -> &str {
        "scala"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("build.sbt"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        if self.is_using_sbt(app) {
            let pkgs = self.get_sbt_dep_pkgs();
            let setup = Phase::setup(Some(pkgs));

            let mut build = Phase::build(None);
            let sbt_exe = self.get_sbt_exe();

            build.add_cmd(format!("{sbt_exe} stage"));
            build.add_cache_directory("/root/.sbt");
            build.add_cache_directory("/root/.ivy2/cache");
            build.add_cache_directory("/root/.cache/coursier");
            build.depends_on_phase("setup");

            let start_cmd = self.get_start_cmd(app).map(StartPhase::new);

            let plan = BuildPlan::new(&vec![setup, build], start_cmd);
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

    fn is_using_sbt(&self, app: &App) -> bool {
        app.includes_file("build.sbt")
    }

    pub fn get_sbt_dep_pkgs(&self) -> Vec<Pkg> {
        let pkgs = vec![self.get_sbt_pkg(), self.get_jdk_pkg()];
        pkgs
    }

    fn get_sbt_pkg(&self) -> Pkg {
        Pkg::new("sbt")
    }

    fn get_jdk_pkg(&self) -> Pkg {
        // sbt uses jdk pkg to compile and package the project
        // already so we should use the same package for the start phase
        // to prevent conflict.
        Pkg::new("jdk")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbt_package() {
        let scala = ScalaProvider {};

        assert!(scala.is_using_sbt(&App::new("examples/scala-sbt").unwrap()));
        assert!(!scala.is_using_sbt(&App::new("examples/node").unwrap()));
    }
}
