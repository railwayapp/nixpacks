use indoc::formatdoc;

use super::plan::phase::Phase;

pub mod pkg;

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
