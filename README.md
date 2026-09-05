# ethos-zero

The ethos schema language, version zero. Ethos specifies the types,
datom fills them with data, and ethos generates the Rust.

## Anatomy

`src/lib.rs` is the ontology: the layers, the kinds, the declaration
types. Each pass is a module named for it.

| pass | kind | from | to |
|---|---|---|---|
| canonicalization | `Canonicalizable` | sweet text | `Canonical`, the braced form, with its seam |
| delineation (protos) | `Protosizable` | canonical text | `Delineation` |
| conception | `Conceivable<File>` | `Delineation`, `Protoform` | `File`, checked whole |
| checking | `Resolving`, `Checkable` | `File` | names resolved, duplicates and undeclared names refused |
| generation | `Generating` | `File` | Rust text |
| datomization | | each declared type | its `Conceivable<Datom>`, `Datomic`, `Incorporable` interactions |
| protosization | `Protosizable`, `Textualizable` | `File` | canonical text (the ascent, cannot fault) |
| actualization | `Actualizable<File>` on `Potential<File>` | text | `File`, or a `Situated<Fault>` |

Every fault carries the path of the structure at fault, in datomic's
path convention, and the actualization situates it in the source
text. Every method call lives under a kind; there are no free
functions, no inherent impls, no closures beyond what std forces, and
no lookup tables: the enums are walked variant by variant.

The crate eats its own food: `fault.ethos` generates `src/fault.rs`
and `ethos-zero.ethos` generates `src/contract.rs`; the freshness test
regenerates both and every fixture under `tests/generated/`.

## File variants

```
Types    [ imports ] [ types ] [ associations ]
Kinds    [ imports ] [ kinds ]
Signal   [ imports ] [ requests ] [ responses ] [ types ]   ; Request and Response implied
Sema     [ imports ] { record positions } [ types ]         ; Record implied
```

An import names a Rust path prefix and the names taken from it;
`std::clone:Clonable.Clone` imports `Clonable` under the source's own
name. A position that reaches its declaring type is boxed, and only
there. `Name.{ T1 T2 }` is a tuple variant.

## CLI

```
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
# -> Generated.[ /abs/out-dir/file.rs ]
```

One inline datom value, no flags; every reply is a value of the
contract's `Response`. With no argument, `ethos-zero` prints its own
ethos.

## Gates

`cargo test` locally; `nix flake check` is the durable gate (build,
test, fmt, clippy, doc, the no-free-functions and no-inherent-methods
guards, and the dependency ethos declarations read by the built tool).
