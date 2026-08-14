{
  description = "ethos-monolith — reads Ethos text, emits Rust per component";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build }:
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
        commonArguments = { inherit src; strictDeps = true; };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in
      {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          test-interface-fixture = craneLib.cargoTest (commonArguments // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test interface_fixture";
          });
          sole-emission-surface = pkgs.runCommand "ethos-monolith-sole-emission-surface" { } ''
            test "$(find ${src}/src -maxdepth 1 -type f -name '*.rs' -printf '%f\n' | sort | tr '\n' ' ')" = "build.rs generate.rs lib.rs "
            test "$(find ${src}/tests -maxdepth 1 -type f -name '*.rs' -printf '%f\n' | sort | tr '\n' ' ')" = "generate.rs interface_fixture.rs "
            grep -q 'pub const ETHOS_GENERATED_MARKER' ${src}/src/generate.rs
            grep -q 'pub struct GeneratedComponent' ${src}/src/generate.rs
            grep -q 'pub struct ComponentGeneration' ${src}/src/generate.rs
            test -f ${src}/fixtures/psyche/interface.ethos
            test -f ${src}/src/fixture/generated.rs
            touch $out
          '';
          binding-law = pkgs.runCommand "ethos-monolith-binding-law" {
            nativeBuildInputs = [ pkgs.ripgrep ];
          } ''
            if rg -n '^impl [A-Za-z_][A-Za-z0-9_]*(<[^>]*>)?[[:space:]]*\{' ${src}/src; then
              echo "inherent production impl methods are forbidden" >&2
              exit 1
            fi
            if rg -n '^(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?fn[[:space:]]' ${src}/src; then
              echo "production free functions are forbidden" >&2
              exit 1
            fi
            if rg -n '^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*;' ${src}/src; then
              echo "production behavior ZSTs are forbidden" >&2
              exit 1
            fi
            touch $out
          '';
          doc = craneLib.cargoDoc (commonArguments // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };
        devShells.default = pkgs.mkShell {
          name = "ethos-monolith";
          packages = [ pkgs.jujutsu toolchain ];
        };
      });
}
