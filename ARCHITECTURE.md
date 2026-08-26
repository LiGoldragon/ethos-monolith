# Architecture

Ethos text is the input. Generated Rust is the output. Consumers commit
the output; they never depend on this crate.

```text
signal.ethos
     │
     ▼
SignalGeneration
     │
     ▼
GeneratedSignal
     │
     ▼
 signal.rs
```

`generate` owns the emission boundary: `SignalGeneration` carries the signal
source and output directory bindings; `GeneratedSignal` carries the one
`GeneratedArtifact` produced by a generation run.

The `signal.ethos` document must contain a source-owned channel binding after
its Interface version header:

```text
Interface.{0 1 0}
Channel.{Orchestrate 1 4}
[]
{
  [Register.PathLock Release.PathLockRelease]
  [PathLockRegistered.PathLock PathLockReleased.PathLockRelease]
  []
  []
  [PathLockName.String PathLockPath.String PathLockPaths.Vector<PathLockPath>]
}
```

`Channel.{Name ContractId WireRevision}` emits the marker `NameWire`, its
`WireContract` binding, typed Datom realization/textualization implementations,
and one
`signal_channel!` declaration. Inputs become its operations; Outputs become
the closed `NameReply`; `NameRequest` aliases its generated operation root.
The binding integers are positive, and an invalid or absent channel declaration
rejects generation before output installation.

`Vector<T>` is the supported collection reference and emits `Vec<T>` when `T`
is a known local type. Imports, trait interactions, unconstrained generic
parameters, and stream runtime declarations remain outside this POC and are
rejected or not selected by the relevant projection; no fallback Rust is
invented for them.

`build` owns the checked-artifact and Cargo metadata contract: consumers
may publish their Ethos source directory through `CargoEthosSourceMetadata`
so that dependents can locate it at build time.

The generator reads plain Ethos text — no authority seal, no bootstrap
pipeline. Its output begins with `ETHOS_GENERATED_MARKER` and is checked into
the consuming repository.

The overnight Interface slice lives under `src/fixture`; its source fixture is
`fixtures/psyche/interface.ethos`. The dialect walks Protos directly in both
directions and projects only complete Signal modules.
`tests/interface_fixture.rs` checks round-trip equality, nested walk evidence,
negative shape witnesses, and the signal projection's Rust syntax.
