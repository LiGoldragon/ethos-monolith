# ethos-monolith

`ethos-monolith` reads Ethos text files and emits Rust — `signal.rs`,
`nexus.rs`, and `sema.rs` — for one component at a time.

Consumers run the generator and commit the emitted Rust into their own
repositories. They do not depend on this crate at build time or runtime.
The generator is invoked through an explicit update step; it is never an
implicit Cargo build-script dependency.

A component call is `ComponentGeneration::new(ethos_directory,
rust_directory).generate()`. It always reads and realizes all three canonical
paths before it installs any output:

```text
ethos_directory/signal.ethos -> rust_directory/signal.rs
ethos_directory/nexus.ethos  -> rust_directory/nexus.rs
ethos_directory/sema.ethos   -> rust_directory/sema.rs
```

`signal.ethos` is channel-bearing. Its `Channel.{Name ContractId
WireRevision}` declaration follows the `Interface` header and owns the public
Signal binding. The generated module supplies `NameWire`, `NameRequest`,
`NameReply`, frame aliases, named payload textual traits, and the complete
`signal_channel!` declaration. `nexus.ethos` and `sema.ethos` are ordinary
Interface documents, including exact empty documents when a component has no
types there.

Named structs and named scalar carriers retain their Ethos heads in their
generated text: `PathLock.{...}` and `PathLockName.value`, respectively.
Consumers parse a concrete payload with `DotosSource::new(text).parse()`;
the channel operation remains the internal wire envelope, not the command-line
text surface.

The durable gate is:

```sh
nix flake check -L
```
