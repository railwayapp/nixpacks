use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{bail, Result};

static DEFAULT_SWIFT_VERSION: &str = "5.4.2";

// From: https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=swift
static AVAILABLE_SWIFT_VERSIONS: &[(&str, &str)] = &[
    ("3.1", "aeaa79dc82980869a88a5955ea3cd3e1944b7d80"),
    ("3.1.1", "8414d8386b9a6b855b291fb3f01a4e3b04c08bbb"),
    ("4.0.3", "2c9d2d65266c2c3aca1e4c80215de8bee5295b04"),
    ("4.1", "92a047a6c4d46a222e9c323ea85882d0a7a13af8"),
    ("4.1.3", "a3962299f14944a0e9ccf8fd84bd7be524b74cd6"),
    ("4.2.1", "7ff8a16f0726342f0a25697867d8c1306d4da7b0"),
    ("4.2.3", "3fa154fd7fed3d6a94322bf08a6def47d6f8e0f6"),
    ("5.0.1", "4599f2bb9a5a6b1482e72521ead95cb24e0aa819"),
    ("5.0.2", "a9eb3eed170fa916e0a8364e5227ee661af76fde"),
    ("5.1.1", "9986226d5182c368b7be1db1ab2f7488508b5a87"),
    ("5.4", "c82b46413401efa740a0b994f52e9903a4f6dcd5"),
    ("5.4.2", "c82b46413401efa740a0b994f52e9903a4f6dcd5"),
];

pub struct SwiftProvider {}

impl Provider for SwiftProvider {
    fn name(&self) -> &str {
        "swift"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Package.swift"))
    }

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let mut setup_phase = SetupPhase::new(vec![
            Pkg::new("coreutils"),
            Pkg::new("swift"),
            Pkg::new("clang"),
            Pkg::new("zlib"),
            Pkg::new("zlib.dev"),
        ]);

        let swift_version = SwiftProvider::get_swift_version(app)?;
        let rev = SwiftProvider::version_number_to_rev(&swift_version);

        if let Some(rev) = rev {
            setup_phase.set_archive(rev);
        } else {
            // Safe to unwrap, "5.4.2" exists on `AVAILABLE_SWIFT_VERSIONS`
            setup_phase.set_archive(
                AVAILABLE_SWIFT_VERSIONS
                    .iter()
                    .find(|(ver, _rev)| *ver == DEFAULT_SWIFT_VERSION)
                    .unwrap()
                    .1
                    .to_string(),
            );
        }

        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("swift package resolve".to_string());

        install_phase.add_file_dependency("Package.swift".to_string());

        if app.includes_file("Package.resolved") {
            install_phase.add_file_dependency("Package.resolved".to_string());
        }

        Ok(Some(install_phase))
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let name = SwiftProvider::get_executable_name(app)?;
        let mut build_phase =
            BuildPhase::new("CC=clang++ swift build -c release --static-swift-stdlib".to_string());
        build_phase.add_cmd(format!(
            "cp ./.build/release/{name} ./{name} && rm -rf ./.build"
        ));
        Ok(Some(build_phase))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        let name = SwiftProvider::get_executable_name(app)?;

        Ok(Some(StartPhase::new(format!("./{}", name))))
    }
}

impl SwiftProvider {
    fn get_swift_version(app: &App) -> Result<String> {
        if app.includes_file(".swift-version") {
            let contents = app.read_file(".swift-version")?;
            let version = contents.split('\n').collect::<Vec<_>>();

            match version.first() {
                Some(v) => Ok(v.trim().to_string()),
                None => bail!("Your .swift-version file is empty"),
            }
        } else if app.includes_file("Package.swift") {
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
        } else {
            Ok(DEFAULT_SWIFT_VERSION.to_string())
        }
    }

    fn get_executable_name(app: &App) -> Result<String> {
        let raw_paths = app.find_files("Sources/**/main.swift")?;
        let paths = raw_paths
            .iter()
            .filter(|&path| !path.to_string_lossy().contains(".build"))
            .collect::<Vec<_>>();

        let path = match paths.first() {
            Some(path) => path.to_string_lossy().to_string(),
            None => bail!("Your swift app doesn't have a main.swift file"),
        };

        let mut names = path.split('/').collect::<Vec<_>>();

        // Safe to unwrap now, path was filtered by glob
        let pos = names.iter().position(|&n| n == "Sources").unwrap();

        names.drain(0..pos);

        Ok(names[1].to_string())
    }

    fn version_number_to_rev(version: &str) -> Option<String> {
        let matched_version = AVAILABLE_SWIFT_VERSIONS
            .iter()
            .find(|(ver, _rev)| *ver == version);

        matched_version.map(|(_ver, rev)| rev.to_string())
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
