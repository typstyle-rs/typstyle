{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-manifest = {
      url = "https://static.rust-lang.org/dist/channel-rust-1.83.0.toml";
      flake = false;
    };
  };

  outputs = inputs@{ flake-parts, crane, nixpkgs, fenix, rust-manifest, self, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = import inputs.systems;

    imports = [
      inputs.flake-parts.flakeModules.easyOverlay
    ];

    perSystem = { self', pkgs, lib, system, ... }:
      let
        cargoToml = lib.importTOML ./Cargo.toml;

        pname = "typstyle";
        version = cargoToml.workspace.package.version;

        rust-toolchain = (fenix.packages.${system}.fromManifestFile rust-manifest).defaultToolchain;

        # Crane-based Nix flake configuration.
        # Based on https://github.com/ipetkov/crane/blob/master/examples/trunk-workspace/flake.nix
        craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;

        # Typstyle files to include in the derivation.
        # Here we include Rust files, docs and tests.
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./crates
            ./tests
          ];
        };

        # Typstyle derivation's args, used within crane's derivation generation
        # functions.
        commonCraneArgs = {
          inherit src pname version;
        };

        # Derivation with just the dependencies, so we don't have to keep
        # re-building them.
        cargoArtifacts = craneLib.buildDepsOnly commonCraneArgs;

        typstyle = craneLib.buildPackage (commonCraneArgs // {
          inherit cargoArtifacts;
          meta.mainProgram = "typstyle";
        });
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        packages = {
          default = typstyle;
          typstyle-dev = self'.packages.default;
        };

        overlayAttrs = builtins.removeAttrs self'.packages [ "default" ];

        apps.default = {
          type = "app";
          program = lib.getExe typstyle;
        };

        checks = {
          typstyle-fmt = craneLib.cargoFmt commonCraneArgs;
          typstyle-clippy = craneLib.cargoClippy (commonCraneArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace -- --deny warnings";
          });
          typstyle-test = craneLib.cargoTest (commonCraneArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--workspace";
          });
        };

        devShells.default = craneLib.devShell {
          checks = self'.checks;
          inputsFrom = [ typstyle ];

          # TODO: Add shiora if/when it is packaged on nixpkgs.
          buildInputs = with pkgs; [
            just
          ];
        };
      };
  };
}
