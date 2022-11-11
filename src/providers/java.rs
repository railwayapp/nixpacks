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
use regex::{Match, Regex};

pub struct JavaProvider {}

impl Provider for JavaProvider {
    fn name(&self) -> &str {
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

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let mut setup: Phase;
        let mut build = if self.is_using_gradle(app) {
            let pkgs = self.get_jdk_and_gradle_pkgs(app)?;
            setup = Phase::setup(Some(pkgs));

            let mut build = Phase::build(None);
            let gradle_exe = self.get_gradle_exe(app);

            // Ensure the gradlew file is executable
            if app.includes_file("./gradlew") && !app.is_file_executable("gradlew") {
                build.add_cmd("chmod +x gradlew");
            }

            build.add_cmd(format!("{} build -x check", gradle_exe));
            build.add_cache_directory("/root/.gradle");
            build
        } else {
            setup = Phase::setup(Some(vec![Pkg::new("jdk")]));
            setup.add_nix_pkgs(&[Pkg::new("maven")]);
            let mvn_exe = self.get_maven_exe(app);
            let mut build = Phase::build(Some(format!("{mvn_exe} -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install", 
                mvn_exe=mvn_exe
            )));
            build.add_cache_directory(".m2/repository");
            build
        };
        let start = StartPhase::new(self.get_start_cmd(app)?);
        build.depends_on = Some(vec!["setup".to_string()]);

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
        let cmd = if self.is_using_gradle(app) {
            format!(
                "java $JAVA_OPTS -jar {} build/libs/*.jar",
                self.get_gradle_port_config(app)?
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

    fn get_gradle_port_config(&self, app: &App) -> Result<String> {
        let file_content = if app.includes_file("build.gradle") {
            app.read_file("build.gradle")?
        } else if app.includes_file("build.gradle.kts") {
            app.read_file("build.gradle.kts")?
        } else {
            String::new()
        };

        let is_spring_boot = file_content.contains("org.springframework.boot:spring-boot")
            || file_content.contains("spring-boot-gradle-plugin")
            || file_content.contains("org.springframework.boot")
            || file_content.contains("org.grails:grails-");

        let port_arg = if is_spring_boot {
            "-Dserver.port=$PORT".to_string()
        } else {
            String::new()
        };

        Ok(port_arg)
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

    pub fn get_jdk_and_gradle_pkgs(&self, app: &App) -> Result<Vec<Pkg>> {
        fn as_default(v: Option<Match>) -> &str {
            match v {
                Some(m) => m.as_str(),
                None => "_",
            }
        }

        let default_pkgs = vec![Pkg::new("jdk"), Pkg::new("gradle")];

        if !app.includes_file("gradle/wrapper/gradle-wrapper.properties") {
            return Ok(default_pkgs);
        }

        let file_content = app.read_file("gradle/wrapper/gradle-wrapper.properties")?;
        let custom_version = Regex::new(r#"(distributionUrl[\S].*[gradle])(-)([0-9|\.]*)"#)?
            .captures(&file_content)
            .map(|c| c.get(3).unwrap().as_str().to_owned());

        // If it's still none, return default
        if custom_version.is_none() {
            return Ok(default_pkgs);
        }

        let custom_version = custom_version.unwrap();
        let matches = Regex::new(r#"^(?:[\sa-zA-Z-"']*)(\d*)(?:\.*)(\d*)(?:\.*\d*)(?:["']?)$"#)?
            .captures(custom_version.as_str().trim());

        // If no matches, just use default
        if matches.is_none() {
            return Ok(default_pkgs);
        }
        let matches = matches.unwrap();
        let parsed_version = as_default(matches.get(1));

        if parsed_version == "_" {
            return Ok(default_pkgs);
        }

        let int_version = parsed_version.parse::<i32>().unwrap_or_default();
        let pkgs = if int_version == 6 {
            vec![Pkg::new("jdk11"), Pkg::new("gradle_6")]
        } else if int_version == 5 {
            vec![Pkg::new("jdk8"), Pkg::new("gradle_5")]
        } else if int_version < 5 {
            vec![Pkg::new("jdk8"), Pkg::new("gradle_4")]
        } else {
            default_pkgs
        };

        Ok(pkgs)
    }
}
