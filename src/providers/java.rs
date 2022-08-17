use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    plan::legacy_phase::{LegacyBuildPhase, LegacySetupPhase, LegacyStartPhase},
};
use anyhow::Result;
pub struct JavaProvider {}

impl Provider for JavaProvider {
    fn name(&self) -> &str {
        "Java"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("pom.xml")
            || app.includes_directory("pom.atom")
            || app.includes_directory("pom.clj")
            || app.includes_directory("pom.groovy")
            || app.includes_file("pom.rb")
            || app.includes_file("pom.scala")
            || app.includes_file("pom.yaml")
            || app.includes_file("pom.yml"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<LegacySetupPhase>> {
        Ok(Some(LegacySetupPhase::new(vec![
            Pkg::new("maven"),
            Pkg::new("jdk8"),
        ])))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<LegacyBuildPhase>> {
        let mvn_exe = self.get_maven_exe(app);
        Ok(Some(LegacyBuildPhase::new(format!("{mvn_exe} -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install", 
            mvn_exe=mvn_exe
        ))))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<LegacyStartPhase>> {
        let start_cmd = self.get_start_cmd(app);
        Ok(Some(LegacyStartPhase::new(start_cmd)))
    }
}

impl JavaProvider {
    fn get_maven_exe(&self, app: &App) -> String {
        // App has a maven wrapper
        if app.includes_file("mvnw") && app.includes_file(".mvn/wrapper/maven-wrapper.properties") {
            "./mvnw".to_string()
        } else {
            "mvn".to_string()
        }
    }

    fn get_start_cmd(&self, app: &App) -> String {
        if app.includes_file("pom.xml") {
            format!(
                "java {} $JAVA_OPTS -jar target/*jar",
                self.get_port_config(app)
            )
        } else {
            "java $JAVA_OPTS -jar target/*jar".to_string()
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
            "".to_string()
        }
    }
}
