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
use path_slash::PathExt;

const DEFAULT_SWIFT_VERSION: &str = "5.8";

// From: https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=swift
const AVAILABLE_SWIFT_VERSIONS: &[(&str, &str)] = &[
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
    ("5.5.2", "7592790b9e02f7f99ddcb1bd33fd44ff8df6a9a7"),
    ("5.5.3", "7cf5ccf1cdb2ba5f08f0ac29fc3d04b0b59a07e4"),
    ("5.6.2", "3c3b3ab88a34ff8026fc69cb78febb9ec9aedb16"),
    ("5.7.3", "8cad3dbe48029cb9def5cdb2409a6c80d3acfe2e"),
    ("5.8", "9957cd48326fe8dbd52fdc50dd2502307f188b0d"),
];

pub struct SwiftProvider {}

impl Provider for SwiftProvider {
    fn name(&self) -> &'static str {
        "swift"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Package.swift"))
    }

    fn get_build_plan(&self, app: &App, _env: &Environment) -> Result<Option<BuildPlan>> {
        let _plan = BuildPlan::default();

        let mut setup = Phase::setup(Some(vec![
            Pkg::new("coreutils"),
            Pkg::new("swift"),
            Pkg::new("clang"),
            Pkg::new("zlib"),
            Pkg::new("zlib.dev"),
        ]));

        let swift_version = SwiftProvider::get_swift_version(app)?;
        let rev = SwiftProvider::version_number_to_rev(&swift_version);

        if let Some(rev) = rev {
            setup.set_nix_archive(rev);
        } else {
            // Safe to unwrap, "5.4.2" exists on `AVAILABLE_SWIFT_VERSIONS`
            setup.set_nix_archive(
                AVAILABLE_SWIFT_VERSIONS
                    .iter()
                    .find(|(ver, _rev)| *ver == DEFAULT_SWIFT_VERSION)
                    .unwrap()
                    .1
                    .to_string(),
            );
        }

        let mut install = Phase::install(Some("swift package resolve".to_string()));
        install.add_file_dependency("Package.swift".to_string());
        if app.includes_file("Package.resolved") {
            install.add_file_dependency("Package.resolved".to_string());
        }

        let name = SwiftProvider::get_executable_name(app)?;
        let mut build = Phase::build(Some(
            "CC=clang++ swift build -c release --static-swift-stdlib".to_string(),
        ));
        build.add_cmd(format!(
            "cp ./.build/release/{name} ./{name} && rm -rf ./.build"
        ));

        let name = SwiftProvider::get_executable_name(app)?;
        let start = StartPhase::new(format!("./{name}"));

        let plan = BuildPlan::new(&vec![setup, install, build], Some(start));

        Ok(Some(plan))
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
                .cloned();

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
            .filter(|&path| !path.to_slash().unwrap().contains(".build"))
            .collect::<Vec<_>>();

        let path = match paths.first() {
            Some(path) => path.to_slash().unwrap().to_string(),
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

        matched_version.map(|(_ver, rev)| (*rev).to_string())
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
