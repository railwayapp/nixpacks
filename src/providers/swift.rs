use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use indoc::formatdoc;

static DEFAULT_SWIFT_VERSION: &str = "5.6.1";

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
        let pkgs = vec![Pkg::new("python38"), Pkg::new("wget")];

        Ok(Some(SetupPhase::new(pkgs)))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let version = SwiftProvider::get_swift_version(&app)?;
        let (download_url, name) = SwiftProvider::make_download_url(version);

        // https://forums.swift.org/t/which-clang-package-should-we-install/20542/14
        let install_cmd = formatdoc! {"
        sudo apt-get update && \
        sudo apt-get install -y build-essential clang libsqlite3-0 libncurses6 libcurl4 libxml2 libatomic1 libedit2 libsqlite3-0 libcurl4 libxml2 libbsd0 libc6-dev && \
        wget -q {download_url} && \
        tar -xf {name}.tar.gz && \
        sudo mv {name} /usr/share/swift
        ",
        name = name,
        download_url = download_url
        };

        let mut install_phase = InstallPhase::new(install_cmd);

        install_phase.add_path("/usr/share/swift/usr/bin".to_string());

        Ok(Some(install_phase))
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
}

impl SwiftProvider {
    fn get_swift_version(app: &App) -> Result<String> {
        if app.includes_file("Package.swift") {
            let contents = app.read_file("Package.swift")?;
            let version = contents
                .split('\n')
                .filter(|&l| l.contains("swift-tools-version:"))
                .map(|l| {
                    l.replace("swift-tools-version:", "")
                        .replace("//", "")
                        .trim()
                        .to_string()
                })
                .collect::<Vec<_>>()
                .first()
                .map(|s| s.to_owned());

            if let Some(version) = version {
                Ok(version)
            } else {
                Ok(DEFAULT_SWIFT_VERSION.to_string())
            }
        } else if app.includes_file(".swift-version") {
            let contents = app.read_file(".swift-version")?;

            Ok(contents.trim().to_string())
        } else {
            Ok(DEFAULT_SWIFT_VERSION.to_string())
        }
    }

    fn make_download_url(version: String) -> (String, String) {
        #[cfg(target_arch = "x86_64")]
        let (download_url, name) = (
            format!("https://download.swift.org/swift-{version}-release/ubuntu2004/swift-{version}-RELEASE/swift-{version}-RELEASE-ubuntu20.04.tar.gz", version = version), 
            format!("swift-{}-RELEASE-ubuntu20.04", version)
        );

        #[cfg(target_arch = "aarch64")]
        let (download_url, name) = (
            format!("https://download.swift.org/swift-{version}-release/ubuntu2004/swift-{version}-RELEASE/swift-{version}-RELEASE-ubuntu20.04-aarch64.tar.gz", version = version), 
            format!("swift-{}-RELEASE-ubuntu20.04-aarch64", version)
        );

        (download_url, name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_custom_version() -> Result<()> {
        assert_eq!(
            &SwiftProvider::get_swift_version(&App::new("./examples/swift-custom-version")?)?,
            "5.4"
        );

        Ok(())
    }
}
