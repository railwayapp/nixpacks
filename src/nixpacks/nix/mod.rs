use indoc::formatdoc;
use std::collections::{BTreeMap, BTreeSet};

use crate::nixpacks::plan::phase::{Phase, Phases};

pub mod pkg;

// This line is automatically updated.
// Last Modified: 2023-01-02 17:04:24 UTC+0000
// https://github.com/NixOS/nixpkgs/commit/293a28df6d7ff3dec1e61e37cc4ee6e6c0fb0847
pub const NIXPKGS_ARCHIVE: &str = "293a28df6d7ff3dec1e61e37cc4ee6e6c0fb0847";

// Version of the Nix archive that uses OpenSSL 1.1
pub const NIXPACKS_ARCHIVE_LEGACY_OPENSSL: &str = "a0b7e70db7a55088d3de0cc370a59f9fbcc906c3";

#[derive(Eq, PartialEq, Default, Debug, Clone)]
struct NixGroup {
    archive: Option<String>,
    pkgs: Vec<String>,
    libs: Vec<String>,
    overlays: Vec<String>,
    files: Vec<String>,
}

fn group_nix_packages_by_archive(phases: &[Phase]) -> Vec<NixGroup> {
    let mut archive_to_packages: BTreeMap<Option<String>, NixGroup> = BTreeMap::new();

    let groups = phases
        .iter()
        .filter(|phase| phase.uses_nix())
        .map(|phase| NixGroup {
            archive: phase.nixpkgs_archive.clone(),
            pkgs: phase.nix_pkgs.clone().unwrap_or_default(),
            libs: phase.nix_libs.clone().unwrap_or_default(),
            overlays: phase.nix_overlays.clone().unwrap_or_default(),
            files: phase.only_include_files.clone().unwrap_or_default(),
        });

    for g in groups {
        match archive_to_packages.get_mut(&g.archive) {
            Some(group) => {
                group.pkgs.extend(g.pkgs);
                group.libs.extend(g.libs);
                group.overlays.extend(g.overlays);
                group.files.extend(g.files);
            }
            None => {
                archive_to_packages.insert(g.archive.clone(), g);
            }
        }
    }

    archive_to_packages
        .values()
        .map(std::clone::Clone::clone)
        .collect()
}

pub fn create_nix_expressions_for_phases(phases: &Phases) -> BTreeMap<String, String> {
    let archive_to_packages = group_nix_packages_by_archive(
        &phases
            .values()
            .map(std::clone::Clone::clone)
            .collect::<Vec<_>>(),
    );

    archive_to_packages
        .iter()
        .fold(BTreeMap::new(), |mut acc, g| {
            acc.insert(nix_file_name(&g.archive), nix_expression_for_group(g));
            acc
        })
}

pub fn nix_file_names_for_phases(phases: &Phases) -> Vec<String> {
    let archives = phases
        .values()
        .filter(|p| p.uses_nix())
        .map(|p| p.nixpkgs_archive.clone())
        .collect::<BTreeSet<_>>();
    archives.iter().map(nix_file_name).collect()
}

pub fn setup_files_for_phases(phases: &Phases) -> Vec<String> {
    let groups = group_nix_packages_by_archive(
        &phases
            .values()
            .map(std::clone::Clone::clone)
            .collect::<Vec<_>>(),
    );

    groups.iter().fold(Vec::new(), |mut acc, g| {
        acc.extend(g.files.clone());
        acc
    })
}

fn nix_file_name(archive: &Option<String>) -> String {
    match archive {
        Some(archive) => format!("nixpkgs-{archive}.nix"),
        None => "nixpkgs.nix".to_string(),
    }
}

fn nix_expression_for_group(group: &NixGroup) -> String {
    let archive = group
        .archive
        .clone()
        .unwrap_or_else(|| NIXPKGS_ARCHIVE.to_string());

    let mut pkgs = group.pkgs.clone();
    pkgs.sort();
    let pkgs = pkgs.join(" ");

    let mut libs = group.libs.clone();
    libs.sort();
    let libs = libs.join(" ");

    let overlays_string = group
        .overlays
        .iter()
        .map(|url| format!("(import (builtins.fetchTarball \"{url}\"))"))
        .collect::<Vec<String>>()
        .join("\n");

    let pkg_import = format!(
        "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{archive}.tar.gz\")"
    );

    // If the openssl library is added, set the OPENSSL_DIR and OPENSSL_LIB_DIR environment variables
    // In the future, we will probably want a generic way for providers to set variables based off Nix package locations
    let openssl_dirs =
        if let Some(openssl_lib) = group.libs.iter().find(|lib| lib.contains("openssl")) {
            formatdoc! {"
          export OPENSSL_DIR=\"${{{openssl_lib}.dev}}\"
          export OPENSSL_LIB_DIR=\"${{{openssl_lib}.out}}/lib\"
        "}
        } else {
            String::new()
        };

    let name = format!("{archive}-env");
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

#[cfg(test)]
mod tests {
    use super::{pkg::Pkg, *};

    #[test]
    fn test_group_nix_packages_by_archive() {
        let mut setup1 = Phase::setup(Some(vec![Pkg::new("foo"), Pkg::new("bar")]));
        setup1.add_pkgs_libs(vec!["lib1".to_string()]);
        setup1.add_file_dependency("test-file".to_string());

        let mut setup2 = Phase::setup(Some(vec![Pkg::new("hello"), Pkg::new("world")]));
        setup2.nixpkgs_archive = Some("archive2".to_string());

        let setup3 = Phase::setup(Some(vec![Pkg::new("baz")]));

        let groups = group_nix_packages_by_archive(&vec![setup1, setup2, setup3]);
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups[0],
            NixGroup {
                archive: None,
                pkgs: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                libs: vec!["lib1".to_string()],
                overlays: vec![],
                files: vec!["test-file".to_string()]
            }
        );
        assert_eq!(
            groups[1],
            NixGroup {
                archive: Some("archive2".to_string()),
                pkgs: vec!["hello".to_string(), "world".to_string()],
                libs: vec![],
                overlays: vec![],
                files: vec![]
            }
        );
    }
}
