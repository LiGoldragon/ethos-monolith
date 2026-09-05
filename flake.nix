{
  description = "ethos-zero — the ethos schema language, version zero";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # The dependencies' own ethos declarations, read by the built tool as a check.
    protos = {
      url = "github:LiGoldragon/protos/2d999f173334";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
    datom-codec = {
      url = "github:LiGoldragon/datom-codec/41a3c073d5c5";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build, protos, datom-codec }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;
        ethosFilter = path: type:
          type == "regular" && pkgs.lib.hasSuffix ".ethos" path;
        src = rust.cleanSource {
          root = ./.;
          extraFilters = [ ethosFilter ];
        };
        # These limits run inside each remote derivation, so cargo, rustc,
        # rustdoc and test children inherit them.  The Nix client limit alone
        # would not constrain the configured remote builder.
        resourcePolicy = ''
          ulimit -v 8388608
        '';
        common = {
          inherit src;
          strictDeps = true;
          preBuild = resourcePolicy;
          preCheck = resourcePolicy;
        };
        cargoArtifacts = craneLib.buildDepsOnly common;
        package = craneLib.buildPackage (common // { inherit cargoArtifacts; });
      in {
        packages.default = package;
        checks = {
          build = craneLib.cargoBuild (common // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (common // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt common;
          clippy = craneLib.cargoClippy (common // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          doc = craneLib.cargoDoc (common // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          dependency-ethos = pkgs.runCommand "ethos-zero-dependency-ethos" {
            inherit package;
            declarations = [
              "${protos}/protos.ethos" "${protos}/protos-kinds.ethos"
              "${datom-codec}/datom-codec.ethos" "${datom-codec}/datom-codec-kinds.ethos"
            ];
          } ''
            ${resourcePolicy}
            ${builtins.readFile ./checks/dependency-ethos.sh}
          '';
          no-free-functions = pkgs.runCommand "ethos-zero-no-free-functions" { inherit src; } ''
            ${resourcePolicy}
            ${builtins.readFile ./checks/no-free-functions.sh}
          '';
          no-inherent-methods = pkgs.runCommand "ethos-zero-no-inherent-methods" { inherit src; } ''
            ${resourcePolicy}
            ${builtins.readFile ./checks/no-inherent-methods.sh}
          '';
        };
        devShells.default = pkgs.mkShell {
          name = "ethos-zero";
          packages = [ pkgs.jujutsu toolchain ];
        };
      });
}
