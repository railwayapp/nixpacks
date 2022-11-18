{ }:

let pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/a0b7e70db7a55088d3de0cc370a59f9fbcc906c3.tar.gz") { overlays = [  ]; };
in with pkgs;
  let
    APPEND_LIBRARY_PATH = "${lib.makeLibraryPath [ stdenv.cc.cc.lib zlib ] }";
    myLibraries = writeText "libraries" ''
      export LD_LIBRARY_PATH="${APPEND_LIBRARY_PATH}:$LD_LIBRARY_PATH"
      
    '';
  in
    buildEnv {
      name = "a0b7e70db7a55088d3de0cc370a59f9fbcc906c3-env";
      paths = [
        (runCommand "a0b7e70db7a55088d3de0cc370a59f9fbcc906c3-env" { } ''
          mkdir -p $out/etc/profile.d
          cp ${myLibraries} $out/etc/profile.d/a0b7e70db7a55088d3de0cc370a59f9fbcc906c3-env.sh
        '')
        gcc pipenv postgresql python311
      ];
    }
