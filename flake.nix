{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      overlays = [ (import rust-overlay) ];
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs {
              system = system;
              overlays = overlays;
            };
          in
          function pkgs
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            })
          ]
          ++ builtins.attrValues {
            inherit (pkgs)
              scdoc
              rust-analyzer-unwrapped
              nixd
              pkg-config
              lua5_4
              libpulseaudio
              ;
          };

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.libpulseaudio}/lib:$LD_LIBRARY_PATH
          '';
        };

      });

      packages = forAllSystems (pkgs: {
        default = pkgs.callPackage ./nix/package.nix {
          rustPlatform =
            let
              rust-bin = pkgs.rust-bin.stable.latest.default;
            in
            pkgs.makeRustPlatform {
              cargo = rust-bin;
              rustc = rust-bin;
            };
        };
      });

      homeManagerModules = {
        default = import ./nix/home-manager.nix;
      };
    };
}
