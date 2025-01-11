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
use anyhow::{bail, Result};
use regex::Regex;

pub struct JavaProvider {}

const DEFAULT_JDK_VERSION: u32 = 17;
const DEFAULT_GRADLE_VERSION: u32 = 8;
const JAVA_NIXPKGS_ARCHIVE: &str = "59dc10b5a6f2a592af36375c68fda41246794b86";

impl Provider for JavaProvider {
    fn name(&self) -> &'static str {
        "java"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("pom.xml")
            || app.includes_directory("pom.atom")
            || app.includes_directory("pom.clj")
            || app.includes_directory("pom.groovy")
            || app.includes_file("pom.rb")
            || app.includes_file("pom.scala")
            || app.includes_file("pom.yaml")
            || app.includes_file("pom.yml")
            || app.includes_file("gradlew"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let (setup, build) = if self.is_using_gradle(app) {
            let pkgs = self.get_jdk_and_gradle_pkgs(app, env)?;
            let mut setup = Phase::setup(Some(pkgs));
            setup.set_nix_archive(JAVA_NIXPKGS_ARCHIVE.to_string());

            let mut build = Phase::build(None);
            let gradle_exe = self.get_gradle_exe(app);

            // Ensure the gradlew file is executable
            if app.includes_file("./gradlew") && !app.is_file_executable("gradlew") {
                build.add_cmd("chmod +x gradlew");
            }

            build.add_cmd(format!("{gradle_exe} clean build -x check -x test"));
            build.add_cache_directory("/root/.gradle");
            build.depends_on_phase("setup");

            (setup, build)
        } else {
            let jdk_version = self.get_jdk_version(app, env)?;
            let jdk_pkg = self.get_jdk_pkg(jdk_version)?;

            let mut setup = Phase::setup(Some(vec![jdk_pkg, Pkg::new("maven")]));
            setup.set_nix_archive(JAVA_NIXPKGS_ARCHIVE.to_string());

            let mvn_exe = self.get_maven_exe(app);
            let mut build = Phase::build(Some(format!("{mvn_exe} -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install"
            )));
            build.add_cache_directory(".m2/repository");
            build.depends_on_phase("setup");

            (setup, build)
        };

        let start = StartPhase::new(self.get_start_cmd(app)?);

        let plan = BuildPlan::new(&vec![setup, build], Some(start));
        Ok(Some(plan))
    }
}

impl JavaProvider {
    fn get_maven_exe(&self, app: &App) -> String {
        // App has a maven wrapper
        if app.includes_file("mvnw") && app.includes_file(".mvn/wrapper/maven-wrapper.properties") {
            "chmod +x ./mvnw && ./mvnw".to_string()
        } else {
            "mvn".to_string()
        }
    }

    fn get_gradle_exe(&self, app: &App) -> String {
        if app.includes_file("gradlew")
            && app.includes_file("gradle/wrapper/gradle-wrapper.properties")
        {
            "./gradlew".to_string()
        } else {
            "gradle".to_string()
        }
    }

    fn get_start_cmd(&self, app: &App) -> Result<String> {
        let build_gradle_content = self.read_build_gradle(app)?;
        let cmd = if self.is_using_gradle(app) {
            format!(
                "java $JAVA_OPTS -jar {} $(ls -1 build/libs/*jar | grep -v plain)",
                self.get_gradle_port_config(&build_gradle_content)
            )
        } else if app.includes_file("pom.xml") {
            format!(
                "java {} $JAVA_OPTS -jar target/*jar",
                self.get_port_config(app)
            )
        } else {
            "java $JAVA_OPTS -jar target/*jar".to_string()
        };

        Ok(cmd)
    }

    fn is_using_gradle(&self, app: &App) -> bool {
        app.includes_file("gradlew")
    }

    fn is_using_spring_boot(&self, build_gradle_content: &str) -> bool {
        build_gradle_content.contains("org.springframework.boot:spring-boot")
            || build_gradle_content.contains("spring-boot-gradle-plugin")
            || build_gradle_content.contains("org.springframework.boot")
            || build_gradle_content.contains("org.grails:grails-")
    }

    fn read_build_gradle(&self, app: &App) -> Result<String> {
        if app.includes_file("build.gradle") {
            app.read_file("build.gradle")
        } else if app.includes_file("build.gradle.kts") {
            app.read_file("build.gradle.kts")
        } else {
            Ok(String::new())
        }
    }

    fn get_gradle_port_config(&self, build_gradle_content: &str) -> String {
        if self.is_using_spring_boot(build_gradle_content) {
            "-Dserver.port=$PORT".to_string()
        } else {
            String::new()
        }
    }

    fn get_port_config(&self, app: &App) -> String {
        let pom_file = app.read_file("pom.xml").unwrap_or_default();
        if pom_file.contains("<groupId>org.wildfly.swarm") {
            "-Dswarm.http.port=$PORT".to_string()
        } else if pom_file.contains("<groupId>org.springframework.boot")
            && pom_file.contains("<artifactId>spring-boot")
        {
            "-Dserver.port=$PORT".to_string()
        } else {
            String::new()
        }
    }

    pub fn get_jdk_and_gradle_pkgs(&self, app: &App, env: &Environment) -> Result<Vec<Pkg>> {
        let gradle_version = self.get_gradle_version(app, env)?;
        let jdk_version = self.get_jdk_version(app, env)?;

        let pkgs = vec![
            self.get_jdk_pkg(jdk_version)?,
            self.get_gradle_pkg(gradle_version)?,
        ];
        Ok(pkgs)
    }

    fn get_jdk_pkg(&self, jdk_version: u32) -> Result<Pkg> {
        let pkg = match jdk_version {
            21 => Pkg::new("jdk21"),
            20 => Pkg::new("jdk20"),
            19 => Pkg::new("jdk"),
            17 => Pkg::new("jdk17"),
            11 => Pkg::new("jdk11"),
            8 => Pkg::new("jdk8"),
            _ => bail!("Unsupported JDK version: {}", jdk_version),
        };

        Ok(pkg)
    }

    fn get_gradle_pkg(&self, gradle_version: u32) -> Result<Pkg> {
        let pkg = match gradle_version {
            8 => Pkg::new("gradle"),
            7 => Pkg::new("gradle_7"),
            6 => Pkg::new("gradle_6"),
            5 => Pkg::new("gradle_5"),
            4 => Pkg::new("gradle_4"),
            _ => bail!("Unsupported Gradle version: {}", gradle_version),
        };

        Ok(pkg)
    }

    fn get_jdk_version(&self, app: &App, env: &Environment) -> Result<u32> {
        // If the JDK version is manually specified, use that
        if let Some(jdk_version) = env.get_config_variable("JDK_VERSION") {
            return Ok(jdk_version.parse::<u32>()?);
        }

        if self.is_using_gradle(app) {
            let gradle_version = self.get_gradle_version(app, env)?;

            // Return a JDK version based on the gradle version
            if gradle_version == 6 {
                return Ok(11);
            } else if gradle_version <= 5 {
                return Ok(8);
            }
        }

        Ok(DEFAULT_JDK_VERSION)
    }

    fn get_gradle_version(&self, app: &App, env: &Environment) -> Result<u32> {
        // If the Gradle version is manually specified, use that
        if let Some(gradle_version) = env.get_config_variable("GRADLE_VERSION") {
            return Ok(gradle_version.parse::<u32>()?);
        }

        if !app.includes_file("gradle/wrapper/gradle-wrapper.properties") {
            return Ok(DEFAULT_GRADLE_VERSION);
        }

        let file_content = app.read_file("gradle/wrapper/gradle-wrapper.properties")?;
        let custom_version = Regex::new(r"(distributionUrl[\S].*[gradle])(-)([0-9|\.]*)")?
            .captures(&file_content)
            .map(|c| c.get(3).unwrap().as_str().to_owned());

        // If it's still none, return default
        if custom_version.is_none() {
            return Ok(DEFAULT_GRADLE_VERSION);
        }

        let custom_version = custom_version.unwrap();
        let matches = Regex::new(r#"^(?:[\sa-zA-Z-"']*)(\d*)(?:\.*)(\d*)(?:\.*\d*)(?:["']?)$"#)?
            .captures(custom_version.as_str().trim());

        let parsed_version = match matches {
            Some(m) => match m.get(1) {
                Some(v) => v.as_str(),
                None => return Ok(DEFAULT_GRADLE_VERSION),
            },
            None => return Ok(DEFAULT_GRADLE_VERSION),
        };

        let int_version = parsed_version.parse::<u32>().unwrap_or_default();
        Ok(int_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_build_gradle_returns_with_content() {
        let java = JavaProvider {};
        let app = App::new("examples/java-gradle-8").unwrap();

        let build_gradle_content = java.read_build_gradle(&app).unwrap();
        let expcted_string = "mavenCentral";

        assert!(build_gradle_content.contains(expcted_string));
    }

    #[test]
    fn test_read_build_gradle_returns_with_content_from_kts_extension() {
        let java = JavaProvider {};
        let app = App::new("examples/java-gradle-8-kotlin").unwrap();

        let build_gradle_content = java.read_build_gradle(&app).unwrap();
        let expcted_string = "mavenCentral";

        assert!(build_gradle_content.contains(expcted_string));
    }

    #[test]
    fn test_read_build_gradle_returns_with_empty_string_if_build_gradle_is_not_found() {
        let java = JavaProvider {};
        let app = App::new("examples/java-maven").unwrap();

        let build_gradle_content = java.read_build_gradle(&app).unwrap();

        assert!(build_gradle_content.is_empty());
    }

    #[test]
    fn test_is_using_spring_boot_returns_true() {
        let java = JavaProvider {};
        let spring_boot_identifiers = vec![
            "org.springframework.boot:spring-boot",
            "spring-boot-gradle-plugin",
            "org.springframework.boot",
            "org.grails:grails-",
        ];

        for spring_boot_identifier in spring_boot_identifiers {
            assert!(java.is_using_spring_boot(spring_boot_identifier));
        }
    }

    #[test]
    fn test_is_using_spring_boot_returns_false() {
        let java = JavaProvider {};

        assert!(!java.is_using_spring_boot(""));
    }

    #[test]
    fn test_get_start_cmd_returns_with_gradle_specific_command() {
        let java = JavaProvider {};
        let app = App::new("examples/java-gradle-hello-world").unwrap();

        let expected_start_cmd =
            String::from("java $JAVA_OPTS -jar  $(ls -1 build/libs/*jar | grep -v plain)");
        assert_eq!(java.get_start_cmd(&app).unwrap(), expected_start_cmd);
    }

    #[test]
    fn test_get_start_cmd_returns_with_maven_specific_command() {
        let java = JavaProvider {};
        let app = App::new("examples/java-maven").unwrap();

        let expected_start_cmd =
            String::from("java -Dserver.port=$PORT $JAVA_OPTS -jar target/*jar");
        assert_eq!(java.get_start_cmd(&app).unwrap(), expected_start_cmd);
    }

    #[test]
    fn test_get_jdk_pkg() {
        let java = JavaProvider {};

        assert_eq!(
            Pkg::new("jdk17"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec![]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("jdk8"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=8"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("jdk21"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=21"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("jdk20"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=20"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("jdk"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_JDK_VERSION=19"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("jdk11"),
            java.get_jdk_pkg(
                java.get_jdk_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_GRADLE_VERSION=6"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_get_gradle_pkg() {
        let java = JavaProvider {};

        assert_eq!(
            Pkg::new("gradle"),
            java.get_gradle_pkg(
                java.get_gradle_version(
                    &App::new("examples/java-gradle-8").unwrap(),
                    &Environment::from_envs(vec![]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("gradle_7"),
            java.get_gradle_pkg(
                java.get_gradle_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec![]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Pkg::new("gradle_5"),
            java.get_gradle_pkg(
                java.get_gradle_version(
                    &App::new("examples/java-gradle-hello-world").unwrap(),
                    &Environment::from_envs(vec!["NIXPACKS_GRADLE_VERSION=5"]).unwrap(),
                )
                .unwrap()
            )
            .unwrap()
        );
    }
}
