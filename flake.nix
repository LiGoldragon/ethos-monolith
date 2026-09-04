{
  description = "ethos-zero — the ethos schema language, version zero";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    protos-map = {
      url = "github:LiGoldragon/protos/56c683ec8d1ec8f8f80e9e12251fbfd08d27d728";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
    datomic-map = {
      url = "github:LiGoldragon/datomic/a27f9b8e778935f1fb2ec0011620cf44a96ea81b";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build, protos-map, datomic-map }:
    flake-utils.lib.eachDefaultSystem (system:
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
        commonArguments = {
          inherit src;
          strictDeps = true;
          ETHOS_PROTOS_MAP = "${protos-map}/protos.ethos";
          ETHOS_DATOMIC_MAP = "${datomic-map}/datomic.ethos";
          ETHOS_PROTOS_RUST = "${protos-map}/src/lib.rs";
          ETHOS_DATOMIC_RUST = "${datomic-map}/src/lib.rs";
          ETHOS_PROTOS_CRATE = "${protos-map}";
          ETHOS_DATOMIC_CRATE = "${datomic-map}";
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          doc = craneLib.cargoDoc (commonArguments // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
        };
        devShells.default = pkgs.mkShell {
          name = "ethos-zero";
          packages = [ pkgs.jujutsu toolchain ];
        };
      });
}
