# ethos-monolith

`ethos-monolith` reads a wire consumer's `signal.ethos` and emits its
`signal.rs` Rust module.

Consumers run the generator and commit the emitted Rust into their own
repositories. They do not depend on this crate at build time or runtime.
The generator is invoked through an explicit update step; it is never an
implicit Cargo build-script dependency.

A signal call is `SignalGeneration::new(ethos_directory,
rust_directory).generate()`. It reads and realizes the one canonical path
before it installs its output:

```text
ethos_directory/signal.ethos -> rust_directory/signal.rs
```

`signal.ethos` is channel-bearing. Its `Channel.{Name ContractId
WireRevision}` declaration follows the `Interface` header and owns the public
Signal binding. The generated module supplies `NameWire`, `NameRequest`,
`NameReply`, frame aliases, typed Datom projection traits, and the complete
`signal_channel!` declaration. The Channel declaration is required.

Datom uses the expected type to select its root. The generated request root is
the channel's `Operation`, so command text starts at the selected operation
variant; fixed records remain positional structural values, and nominal Rust
wrappers carry no ad hoc textual heads. Consumers use `DatomText<Operation>`
and `DatomRoot` to realize and textualize the generated contract. The channel
operation remains the internal wire envelope, not a second text shape.

The durable gate is:

```sh
nix flake check -L
```
