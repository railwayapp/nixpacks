{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
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

      outputsBuilder = channels: with channels;
        let
          package = with nixpkgs; rustPlatform.buildRustPackage {
            pname = "nixpacks";
            version = "1.33.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            # For tooling like rust-analyzer
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            doCheck = false;
            meta = with nixpkgs.lib; {
              description = "App source + Nix packages + Docker = Image";
              homepage = "https://github.com/railwayapp/nixpacks";
              license = licenses.mit;
              maintainers = [ maintainers.zoedsoupe ];
            };
          };
        in {
          packages = {
            nixpacks = package;
            default = package;
          };

          devShells = {
            nixpacks = package;
            default = package;
          };

          checks = {
            nixpacks = package;
          };
        };
    };
}
