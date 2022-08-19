{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs@{ self, utils, rust-overlay, ... }:
    utils.lib.mkFlake rec {
      inherit self inputs;

      supportedSystems = [
        "aarch64-linux"
        "aarch64-darwin"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      sharedOverlays = [ (import rust-overlay) ];

      outputsBuilder = channels: with channels; {
        packages = with nixpkgs; { 
          inherit (nixpkgs) package-from-overlays;

          nixpacks = rustPlatform.buildRustPackage {
            pname = "nixpacks";
            version = "v0.3.0";
            doCheck = true;
            src = ./.;
            checkInputs = [ rustfmt clippy ];
            # skip `cargo test` due tests FHS dependency
            checkPhase = ''
              runHook preCheck

              cargo check
              rustfmt --check src/**/*.rs
              cargo clippy

              runHook postCheck
            '';
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            meta = with nixpkgs.lib; {
              description = "App source + Nix packages + Docker = Image";
              homepage = "https://github.com/railwayapp/nixpacks";
              license = licenses.mit;
              maintainers = [ maintainers.zoedsoupe ];
            };
          };
        };

        devShell = nixpkgs.mkShell {
          name = "nixpacks";

          buildInputs = with nixpkgs; [
            # rust overlay already comes with complete toolchains
            # see more at https://github.com/oxalica/rust-overlay
            rust-bin.stable.latest.complete docker
          ];
        };
      };
    };
}
