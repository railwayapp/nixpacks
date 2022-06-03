use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use indoc::formatdoc;

pub struct SwiftProvider {}

impl Provider for SwiftProvider {
    fn name(&self) -> &str {
        "swift"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Package.swift")
            || (app.includes_file("Package.swift") && app.includes_file("Package.resolved")))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let pkgs = vec![
            Pkg::new("clang_13"),
            Pkg::new("python27Full"),
            Pkg::new("wget"),
        ];

        Ok(Some(SetupPhase::new(pkgs)))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        #[cfg(target_arch = "x86_64")]
        let (download_url, name) = (
            "https://download.swift.org/swift-5.6.1-release/ubuntu2004/swift-5.6.1-RELEASE/swift-5.6.1-RELEASE-ubuntu20.04.tar.gz", 
            "swift-5.6.1-RELEASE-ubuntu20.04"
        );

        #[cfg(target_arch = "aarch64")]
        let (download_url, name) = (
            "https://download.swift.org/swift-5.6.1-release/ubuntu2004-aarch64/swift-5.6.1-RELEASE/swift-5.6.1-RELEASE-ubuntu20.04-aarch64.tar.gz", 
            "swift-5.6.1-RELEASE-ubuntu20.04-aarch64"
        );

        let install_cmd = formatdoc! {"
        wget {download_url} && \
        tar -xf {name}.tar.gz && \
        sudo mv {name} /usr/share/swift
        ",
        name=name,
        download_url=download_url
        };

        Ok(Some(InstallPhase::new(install_cmd)))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new(
            "swift build -c release --static-swift-stdlib".to_string(),
        )))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let raw_paths = app.find_files("Sources/**/main.swift")?;
        let paths = raw_paths
            .iter()
            .filter(|&path| !path.to_string_lossy().contains(".build"))
            .collect::<Vec<_>>();

        let path = match paths.first() {
            Some(path) => path.to_string_lossy().to_string(),
            None => return Ok(None),
        };

        let mut names = path.split('/').collect::<Vec<_>>();

        // Safe to unwrap, because the path was filtered by the glob expression
        let pos = names.iter().position(|&n| n == "Sources").unwrap();

        names.drain(0..pos);

        Ok(Some(StartPhase::new(format!(
            "./.build/release/{}",
            names[1]
        ))))
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        let mut variables = EnvironmentVariables::default();

        variables.insert(
            "PATH".to_string(),
            "/usr/share/swift/usr/bin:$PATH".to_string(),
        );

        Ok(Some(variables))
    }
}
