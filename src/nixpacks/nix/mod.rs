use super::plan::{new_build_plan::NewPhase, BuildPlan};
use indoc::formatdoc;

pub mod pkg;

pub fn create_nix_expression(phase: &NewPhase) -> String {
    let pkgs = phase.nix_pkgs.clone().unwrap_or_default();

    let nixpkgs = pkgs
        .iter()
        .map(|p| p.to_nix_string())
        .collect::<Vec<String>>()
        .join(" ");

    let libraries = phase.nix_libraries.clone().unwrap_or_default().join(" ");

    let nix_archive = phase.nixpacks_archive.clone();
    let pkg_import = match nix_archive {
        Some(archive) => format!(
            "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
            archive
        ),
        None => "import <nixpkgs>".to_string(),
    };

    let mut overlays: Vec<String> = Vec::new();
    for pkg in &pkgs {
        if let Some(overlay) = &pkg.overlay {
            overlays.push(overlay.to_string());
        }
    }

    let overlays_string = overlays
        .iter()
        .map(|url| format!("(import (builtins.fetchTarball \"{}\"))", url))
        .collect::<Vec<String>>()
        .join("\n");

    let nix_expression = formatdoc! {"
            {{ }}:

            let pkgs = {pkg_import} {{ overlays = [ {overlays_string} ]; }};
            in with pkgs;
              let
                APPEND_LIBRARY_PATH = \"${{lib.makeLibraryPath [ {libraries} ] }}\";
                myLibraries = writeText \"libraries\" ''
                  export LD_LIBRARY_PATH=\"${{APPEND_LIBRARY_PATH}}:$LD_LIBRARY_PATH\"
                '';
              in
                buildEnv {{
                  name = \"env\";
                  paths = [
                    (runCommand \"libraries\" {{ }} ''
                      mkdir -p $out/etc/profile.d
                      cp ${{myLibraries}} $out/etc/profile.d/libraries.sh
                    '')
                    {nixpkgs}
                  ];
                }}
        "};

    nix_expression
}
