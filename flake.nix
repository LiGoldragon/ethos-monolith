{
  description = "ethos-zero — File anatomy over Protos Portion";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    protos-map = {
      url = "github:LiGoldragon/protos/bfde3b878dd3de2991d7f605b59f57a13ef8f20b";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
    datomic-map = {
      url = "github:LiGoldragon/datomic/b670c72d0c2cb94ad1e39b372271f6569d91e214";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
    signal-orchestrate-interface = {
      url = "github:LiGoldragon/signal-orchestrate/6fc8c5b7f1880b73461a4ffa863a3f8952245c0a";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
    meta-signal-orchestrate-interface = {
      url = "github:LiGoldragon/meta-signal-orchestrate/d4dd208cd6e10254075a0c311a8e8a14a1ff3f8d";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-build.follows = "rust-build";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build, protos-map, datomic-map
    , signal-orchestrate-interface, meta-signal-orchestrate-interface }:
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
          ETHOS_SIGNAL_ORCHESTRATE_SOURCE = "${signal-orchestrate-interface}/ethos/signal.ethos";
          ETHOS_META_SIGNAL_ORCHESTRATE_SOURCE = "${meta-signal-orchestrate-interface}/ethos/signal.ethos";
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
