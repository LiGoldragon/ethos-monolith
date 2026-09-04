# ethos-zero

The ethos schema language, version zero. Reads an ethos file and emits
committed Rust by constructing `syn::File` with `quote`.

## Roots

A **Library** declares types, kinds (traits), and associations:

```
Library.{0 1 0}
[ protos:Text ]
[ Sink.{ Text Vector<Text> }
  SinkError.[ Closed Full ]
  Roles.« Text Integer » ]
[ Summarizable.[ summarize.[ Text ] ]
  Fillable.[ push!{ [ Text ] [ Result<Integer SinkError> ] }
             drain![ Vector<Text> ]
             create:[ Self ] ] ]
[ Sink.[ Summarizable Fillable ] ]
```

Target Rust:

```rust
pub struct Sink(pub protos::Text, pub Vec<protos::Text>);
pub enum SinkError { Closed, Full }
pub type Roles = std::collections::BTreeMap<protos::Text, i64>;

pub trait Summarizable { fn summarize(&self) -> protos::Text; }
pub trait Fillable {
    fn push(&mut self, input_0: protos::Text) -> Result<i64, SinkError>;
    fn drain(&mut self) -> Vec<protos::Text>;
    fn create() -> Self;
}
```

A **Signal** declares request/response types and a wire module:

```
Signal.{1 0 0}
[]
[ Lock.LockRequest Release.LockId Observe.ObserveSelection ]
[ Locked.Lock Released.Lock Observed.Observation ]
[ LockId.Integer LockRequest.{ LockName FlowId LockPaths LockReason } ... ]
```

Target Rust includes rkyv-derived types, `Request`/`Reply` enums, and a
`Frame`/`Body`/`Refusal` wire envelope.

## Intrinsic names

`Text`, `Integer`, `Decimal`, `Boolean`, `Meaning`, `Vector`, `Option`,
`Result`, `Self` need no import. An explicit `protos:Text` import means the
same.

## CLI

```
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
```

Prints `Generated.[ /abs/out-dir/file.rs ]` or `GenerationFailure.{ path reason }`.
With no argument, prints its own ethos (self-description).

## Testing

Run the durable gate with `nix flake check -L` (via configured remote builder)
and the fast local witness with `cargo test`.
