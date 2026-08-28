# ethos-zero

`ethos-zero` embodies canonical headed Ethos Files through Protos `Portion`
anatomy and emits Rust only by constructing `syn::File` with `quote`.

The public roots are `Interface.{0 1 0}` and `Schema.{0 1 0}`. External
imports resolve only through a supplied Datomic-backed manifest; there is no
filesystem or legacy-parser fallback.

Run the durable gate with `nix flake check -L` and the fast local witness with
`cargo test`.
