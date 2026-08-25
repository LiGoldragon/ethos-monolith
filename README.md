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
`NameReply`, frame aliases, named payload textual traits, and the complete
`signal_channel!` declaration. The Channel declaration is required.

Named structs and named scalar carriers retain their Ethos heads when used as
concrete values: `PathLock.{...}` and `PathLockName.value`, respectively.
Within a named record the schema already determines each field type, so the
generated text recursively uses the underlying field values: a lock is
`PathLock.{name [/absolute/path] (description)}`, rather than repeating every
nominal field head.
Consumers parse a concrete payload with `DotosSource::new(text).parse()`;
the channel operation remains the internal wire envelope, not the command-line
text surface.

The durable gate is:

```sh
nix flake check -L
```
