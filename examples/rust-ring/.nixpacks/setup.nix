{ }:

let pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.tar.gz") { overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ]; };
in with pkgs;
  let
    APPEND_LIBRARY_PATH = "${lib.makeLibraryPath [  ] }";
    myLibraries = writeText "libraries" ''
      export LD_LIBRARY_PATH="${APPEND_LIBRARY_PATH}:$LD_LIBRARY_PATH"
      
    '';
  in
    buildEnv {
      name = "setup-env";
      paths = [
        (runCommand "setup-env" { } ''
          mkdir -p $out/etc/profile.d
          cp ${myLibraries} $out/etc/profile.d/setup-env.sh
        '')
        gcc (rust-bin.stable.latest.default.override { targets = ["aarch64-unknown-linux-musl"]; })
      ];
    }
