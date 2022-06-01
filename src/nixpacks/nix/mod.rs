use super::plan::BuildPlan;
use indoc::formatdoc;

pub mod pkg;

pub fn create_nix_expression(plan: &BuildPlan) -> String {
    let setup_phase = plan.setup.clone().unwrap_or_default();

    let nixpkgs = setup_phase
        .pkgs
        .iter()
        .map(|p| p.to_nix_string())
        .collect::<Vec<String>>()
        .join(" ");

    let nix_archive = setup_phase.archive.clone();
    let pkg_import = match nix_archive {
        Some(archive) => format!(
            "import (fetchTarball \"https://github.com/NixOS/nixpkgs/archive/{}.tar.gz\")",
            archive
        ),
        None => "import <nixpkgs>".to_string(),
    };

    let mut overlays: Vec<String> = Vec::new();
    for pkg in &setup_phase.pkgs {
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

            let
              pkgs = {pkg_import} {{ 
                overlays = [
                  {overlays}
                ];
              }};
            in with pkgs;
            buildEnv {{
              name = \"env\";
              paths = [
                {pkgs}
              ];
            }}
        ",
    pkg_import=pkg_import,
    pkgs=nixpkgs,
    overlays=overlays_string};

    nix_expression
}
