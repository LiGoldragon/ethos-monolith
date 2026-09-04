# Architecture

```text
Potential  --Actualizing::actualize-->  Concept  --Emitting::emit-->  Rust text
  |                                       |
  text -> delineate -> portions           concept -> quote -> syn::File -> text
```

ethos-zero has no character parser. The reader asks structural questions of
Protos protoforms. The emitter constructs syntax nodes with `quote` and
validates them as `syn::File`; it does not assemble Rust source strings.
Every method call lives under a trait.

## Entry kinds

Declared in ethos-zero.ethos:

```
Actualizing.[ actualize.[ Result<Concept Fault> ] ]
Emitting.[ emit.[ Result<Text Fault> ] ]
```

Target Rust:

```rust
pub trait Actualizing {
    fn actualize(&self) -> Result<Concept, Fault>;
}

pub trait Emitting {
    fn emit(&self) -> Result<String, Fault>;
}

impl Actualizing for Potential { /* delineate, then walk portions */ }
impl Emitting for Concept { /* quote tokens, validate as syn::File */ }
```

Usage:

```rust
use ethos_zero::{Potential, Actualizing, Emitting};

let concept = Potential::from(source).actualize()?;
let rust = concept.emit()?;
```

`Potential` wraps a string that may be an ethos file. `Actualizing::actualize`
descends from text to concept; `Emitting::emit` descends from concept to the
generated Rust (the corporal layer).

## Concept types

The reader produces one of two concept types:

| Root | Sections | Rust target |
|---|---|---|
| `Library.{ver}` | imports, types, kinds, associations | data types, traits, association assertions |
| `Signal.{ver}` | imports, requests, responses, types | data types with rkyv derives, Request/Reply enums, Frame/Body/Refusal wire envelope |

Type declarations:

| Ethos | Rust |
|---|---|
| `Name.{ T1 T2 }` | `pub struct Name(pub T1, pub T2);` (tuple struct) |
| `Name.[ V1 V2 ]` | `pub enum Name { V1, V2 }` |
| `Name.Type` | `pub type Name = Type;` |
| `Name.\u{00AB} K V \u{00BB}` | `pub type Name = BTreeMap<K, V>;` |

Kind declarations (traits):

| Ethos | Rust |
|---|---|
| `Name.[ cap.[ T ] ]` | `pub trait Name { fn cap(&self) -> T; }` |
| `Name.{ [S] [A<B>] \u{00AB} C T \u{00BB} [cap![ T ]] }` | `pub trait Name: S { type A: B; const C: T; fn cap(&mut self) -> T; }` |

Intrinsic names are fully qualified: `Integer` emits as `protos::Integer`,
`Decimal` as `protos::Decimal`, `Boolean` as `protos::Boolean`,
`Text` as `protos::Text`, `Symbol` as `protos::Symbol`,
`Meaning` as `datomic::Meaning`.

## Layers

```text
Text -> Protoform -> Concept -> Corporal
  Structural::delineate  Conceptual::conceive  Datomic::incorporate
                         <-  Protosizable::protosize  <-  Datomic::datomize
```

Every declared type gets `impl datomic::Datomic` generated from its anatomy.
Tuple structs incorporate from braced protoforms by position; enums incorporate
from bare symbols (unit variants) or headed protoforms (typed variants).

## File forms

The sweet form (outer braces implied in a file):

```
Library.{0 1 0}
[imports]
[types]
[kinds]
[associations]
```

The full form:

```
Library.{ {0 1 0} [imports] [types] [kinds] [associations] }
```

## Wire (Signal only)

For a Signal root, all declared types carry rkyv derives. Three envelope types
are generated:

```
Frame.{ Version Body }
Body.[ Request.Request Reply.Reply Refusal.Refusal ]
Refusal.[ VersionMismatch.{ Version Version } Unreadable ]
```

The contract id is gone (one contract per socket). The Signal's version is the
wire version.

## CLI

`ethos-zero` is a direct datom tool:

```
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
-> Generated.[ /abs/out-dir/file.rs ]

ethos-zero
-> (prints ethos-zero.ethos)
```
