# ethos-zero

`ethos-zero` embodies canonical headed Ethos Files through Protos `Portion`
anatomy and emits Rust only by constructing `syn::File` with `quote`.

The public roots are `Interface.{0 1 0}` and `Schema.{0 1 0}`. External
imports resolve only through a supplied Datomic-backed manifest; there is no
filesystem or legacy-parser fallback.

Schema trait defaults are structural and map-owned. A method default is a
chain beginning at `Self`, such as
`Default.Chain.[Self toPortion print.[Layout.Flat]]`; each bare subsequent
term calls a no-argument Rust method and a bracketed term supplies structural
path arguments. `Default.Yes` is not a valid default body.

Run the durable gate with `nix flake check -L` and the fast local witness with
`cargo test`.
