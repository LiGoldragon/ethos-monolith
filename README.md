# ethos-rust

`ethos-rust` reads Ethos text files and emits Rust — `signal.rs`,
`nexus.rs`, and `sema.rs` — for one component at a time.

Consumers run the generator and commit the emitted Rust into their own
repositories. They do not depend on this crate at build time or runtime.
The generator is invoked through an explicit update step; it is never an
implicit Cargo build-script dependency.

The durable gate is:

```sh
nix flake check -L
```
