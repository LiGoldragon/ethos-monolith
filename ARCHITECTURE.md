# Architecture

Ethos text is the input. Generated Rust is the output. Consumers commit
the output; they never depend on this crate.

```text
signal.ethos  nexus.ethos  sema.ethos
         \         |         /
          ComponentGeneration
                  │
                  ▼
          GeneratedComponent
         /         |         \
    signal.rs  nexus.rs  sema.rs
```

`generate` owns the emission boundary: `ComponentGeneration` carries the
source and output directory bindings; `GeneratedComponent` carries the
three `GeneratedArtifact` values produced by one generation run.

`build` owns the checked-artifact and Cargo metadata contract: consumers
may publish their Ethos source directory through `CargoEthosSourceMetadata`
so that dependents can locate it at build time.

The generator reads plain Ethos text — no authority seal, no bootstrap
pipeline. Each of the three output files begins with
`ETHOS_GENERATED_MARKER` and is checked into the consuming repository.

The overnight Interface slice lives under `src/fixture`: the source fixture
is `fixtures/psyche/interface.ethos`, and its committed Rust projection is
`src/fixture/generated.rs`. The dialect walks Protos directly in both
directions. `tests/interface_fixture.rs` checks round-trip equality, nested
walk evidence, negative shape witnesses, generated freshness, and use of the
projection as a real Rust module.
