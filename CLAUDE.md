ethos-zero reads ethos files (Library and Signal roots) and emits Rust.

The public API is `ethos_zero::read` (text to Concept) and `ethos_zero::emit`
(Concept to Rust). The CLI `ethos-zero` speaks datom: one inline value, no
flags.

Run the durable gate with `nix flake check -L` (via configured remote builder)
and the fast local witness with `cargo test`.
