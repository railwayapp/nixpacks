use super::plan::BuildPlan;
use indoc::formatdoc;

pub mod pkg;

pub fn create_nix_expression(plan: &BuildPlan) -> String {
    let setup_phase = plan.setup.clone().unwrap_or_default();

    let nixpkgs = setup_phase
        .pkgs
        .iter()
        .map(pkg::Pkg::to_nix_string)
        .collect::<Vec<String>>()
        .join(" ");

    let libraries = setup_phase.libraries.unwrap_or_default().join(" ");

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

    let nix_expression = formatdoc! {"
            {{ }}:

            let pkgs = {pkg_import} {{ overlays = [ {overlays_string} ]; }};
            in with pkgs;
              let
                APPEND_LIBRARY_PATH = \"${{lib.makeLibraryPath [ {libraries} ] }}\";
                myLibraries = writeText \"libraries\" ''
                  export LD_LIBRARY_PATH=\"${{APPEND_LIBRARY_PATH}}:$LD_LIBRARY_PATH\"
                  {openssl_dirs}
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
