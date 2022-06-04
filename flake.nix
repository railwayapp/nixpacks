{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
  };

  outputs = inputs@{ self, utils, ... }:
    utils.lib.mkFlake rec {
      inherit self inputs;

      supportedSystems = [
        "aarch64-linux"
        "aarch64-darwin"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      outputsBuilder = channels: with channels; {
        packages = with nixpkgs; { 
          inherit (nixpkgs) package-from-overlays;

          nixpacks = rustPlatform.buildRustPackage rec {
            pname = "nixpacks";
            version = "v0.0.20";
            doCheck = true;
            src = ./.;
            nativeBuildInputs = [ clippy ];
            # skip `cargo test` due tests FHS dependency
            checkPhase = ''
              cargo check
              cargo clippy
            '';
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            meta = with nixpkgs.lib; {
              description = "App source + Nix packages + Docker = Image Resources";
              homepage = "https://github.com/railwayapp/nixpacks";
              license = licenses.mit;
              maintainers = [ maintainers.zoedsoupe ];
            };
          };
        };

        devShell = nixpkgs.mkShell {
          name = "nixpacks";

          buildInputs = with nixpkgs; [
            rustc cargo rustfmt clippy docker
          ];
        };
      };
    };
}
