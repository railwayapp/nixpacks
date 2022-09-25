use super::plan::phase::{Phase, Phases};
use indoc::formatdoc;
use itertools::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub mod pkg;

// This line is automatically updated.
// Last Modified: 2022-09-12 17:11:54 UTC+0000
// https://github.com/NixOS/nixpkgs/commit/a0b7e70db7a55088d3de0cc370a59f9fbcc906c3
pub const NIXPKGS_ARCHIVE: &str = "a0b7e70db7a55088d3de0cc370a59f9fbcc906c3";

#[derive(Debug, Clone)]
struct NixGroup {
    archive: Option<String>,
    pkgs: Vec<String>,
    libs: Vec<String>,
    overlays: Vec<String>,
}

fn group_nix_packages_by_archive(phases: &Vec<Phase>) -> Vec<NixGroup> {
    // Mapping from nixpkgs archive to (packages, libraries)
    let mut archive_to_packages: BTreeMap<Option<String>, NixGroup> = BTreeMap::new();

    let groups = phases
        .clone()
        .into_iter()
        .filter(|phase| phase.uses_nix())
        .map(|phase| NixGroup {
            archive: phase.nixpkgs_archive,
            pkgs: phase.nix_pkgs.unwrap_or_default(),
            libs: phase.nix_libs.unwrap_or_default(),
            overlays: phase.nix_overlays.unwrap_or_default(),
        });

    for g in groups {
        match archive_to_packages.get_mut(&g.archive) {
            Some(group) => {
                group.pkgs.extend(g.pkgs);
                group.libs.extend(g.libs);
                group.overlays.extend(g.overlays);
            }
            None => {
                archive_to_packages.insert(g.archive.clone(), g);
            }
        }
    }

    archive_to_packages.values().map(|g| g.clone()).collect()
}

pub fn create_nix_expressions_for_phases(phases: &Phases) -> BTreeMap<String, String> {
    let archive_to_packages =
        group_nix_packages_by_archive(&phases.values().map(|p| p.clone()).collect());

    archive_to_packages
        .iter()
        .fold(BTreeMap::new(), |mut acc, g| {
            acc.insert(nix_file_name(&g.archive), nix_expression_for_group(g));
            acc
        })
}

pub fn nix_file_names_for_phases(phases: &Phases) -> Vec<String> {
    let archives = BTreeSet::from_iter(
        phases
            .values()
            .filter(|p| p.uses_nix())
            .map(|p| p.nixpkgs_archive.clone()),
    );
    archives.iter().map(|a| nix_file_name(a)).collect()
}

fn nix_file_name(archive: &Option<String>) -> String {
    match archive {
        Some(archive) => format!("nixpkgs-{}.nix", archive),
        None => "nixpkgs.nix".to_string(),
    }
}

fn nix_expression_for_group(group: &NixGroup) -> String {
    let archive = group
        .archive
        .clone()
        .unwrap_or_else(|| NIXPKGS_ARCHIVE.to_string());
    let pkgs = group.pkgs.clone().into_iter().join(" ");
    let libs = group.libs.clone().into_iter().join(" ");
    let overlays_string = group
        .overlays
        .iter()
        .map(|url| format!("(import (builtins.fetchTarball \"{}\"))", url))
        .collect::<Vec<String>>()
        .join("\n");

    let pkg_import = format!(
        "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
        archive
    );

    // If the openssl library is added, set the OPENSSL_DIR and OPENSSL_LIB_DIR environment variables
    // In the future, we will probably want a generic way for providers to set variables based off Nix package locations
    let openssl_dirs = if libs.contains("openssl") {
        formatdoc! {"
          export OPENSSL_DIR=\"${{openssl.dev}}\"
          export OPENSSL_LIB_DIR=\"${{openssl.out}}/lib\"
        "}
    } else {
        String::new()
    };

    let name = format!("{}-env", archive);
    let nix_expression = formatdoc! {"
            {{ }}:

            let pkgs = {} {{ overlays = [ {} ]; }};
            in with pkgs;
              let
                APPEND_LIBRARY_PATH = \"${{lib.makeLibraryPath [ {} ] }}\";
                myLibraries = writeText \"libraries\" ''
                  export LD_LIBRARY_PATH=\"${{APPEND_LIBRARY_PATH}}:$LD_LIBRARY_PATH\"
                  {}
                '';
              in
                buildEnv {{
                  name = \"{name}\";
                  paths = [
                    (runCommand \"{name}\" {{ }} ''
                      mkdir -p $out/etc/profile.d
                      cp ${{myLibraries}} $out/etc/profile.d/{name}.sh
                    '')
                    {}
                  ];
                }}
        ",
        pkg_import,
        overlays_string,
        libs,
        openssl_dirs,
        pkgs,
        name=name,
    };

    nix_expression
}

pub fn create_nix_expression(phase: &Phase) -> String {
    let nixpkgs = phase.nix_pkgs.clone().unwrap_or_default().join(" ");

    let libraries = phase.nix_libs.clone().unwrap_or_default().join(" ");

    let nix_archive = phase.nixpkgs_archive.clone();
    let pkg_import = match nix_archive {
        Some(archive) => format!(
            "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
            archive
        ),
        None => "import <nixpkgs>".to_string(),
    };

    let overlays = phase.nix_overlays.clone().unwrap_or_default();

    let overlays_string = overlays
        .iter()
        .map(|url| format!("(import (builtins.fetchTarball \"{}\"))", url))
        .collect::<Vec<String>>()
        .join("\n");

    // If the openssl library is added, set the OPENSSL_DIR and OPENSSL_LIB_DIR environment variables
    // In the future, we will probably want a generic way for providers to set variables based off Nix package locations
    let openssl_dirs = if libraries.contains("openssl") {
        formatdoc! {"
          export OPENSSL_DIR=\"${{openssl.dev}}\"
          export OPENSSL_LIB_DIR=\"${{openssl.out}}/lib\"
        "}
    } else {
        String::new()
    };

    let name = format!("{}-env", phase.get_name());
    let nix_expression = formatdoc! {"
            {{ }}:

            let pkgs = {} {{ overlays = [ {} ]; }};
            in with pkgs;
              let
                APPEND_LIBRARY_PATH = \"${{lib.makeLibraryPath [ {} ] }}\";
                myLibraries = writeText \"libraries\" ''
                  export LD_LIBRARY_PATH=\"${{APPEND_LIBRARY_PATH}}:$LD_LIBRARY_PATH\"
                  {}
                '';
              in
                buildEnv {{
                  name = \"{name}\";
                  paths = [
                    (runCommand \"{name}\" {{ }} ''
                      mkdir -p $out/etc/profile.d
                      cp ${{myLibraries}} $out/etc/profile.d/{name}.sh
                    '')
                    {}
                  ];
                }}
        ",
        pkg_import,
        overlays_string,
        libraries,
        openssl_dirs,
        nixpkgs,
        name=name,
    };

    nix_expression
}

#[cfg(test)]
mod tests {
    use super::{pkg::Pkg, *};

    #[test]
    fn test_group_nix_packages_by_archive() {
        let mut setup1 = Phase::setup(Some(vec![Pkg::new("foo"), Pkg::new("bar")]));
        setup1.add_pkgs_libs(vec!["lib1".to_string()]);

        let mut setup2 = Phase::setup(Some(vec![Pkg::new("hello"), Pkg::new("world")]));
        setup2.nixpkgs_archive = Some("archive2".to_string());

        let setup3 = Phase::setup(Some(vec![Pkg::new("baz")]));

        let groups = group_nix_packages_by_archive(&vec![setup1, setup2, setup3]);
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups.get(&None).unwrap(),
            &(
                vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                vec!["lib1".to_string()]
            )
        );
        assert_eq!(
            groups.get(&Some("archive2".to_string())).unwrap(),
            &(vec!["hello".to_string(), "world".to_string()], vec![])
        );
    }
}
