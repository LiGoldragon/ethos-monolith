# Upgrades

## 1.0.0 — ProtoformStack rewrite

ethos-zero is rewritten from scratch. The Interface/Schema roots, Channel,
Visibility vocabulary, manifest resolution, named struct fields, the Nexus
runtime, and both edge CLIs are removed. The replacement is the ethos schema
language described in Vision/ethos.md.

### Roots

| Old | New |
|---|---|
| `Interface.{ver} Channel.{name contract wire} [imports] { [in] [out] [refusal] [stream] [types] }` | `Signal.{ver} [imports] [requests] [responses] [types]` |
| `Schema.{ver} [imports] Types.[ ] Kinds.[ ] Associations.[ ]` | `Library.{ver} [imports] [types] [kinds] [associations]` |

The sweet form (outer braces implied in a file) is the default for both roots.
The full form (`Root.{ {ver} sections }`) and a bracket of several ethos
objects are also accepted.

### Type declarations

All struct types are positional (tuple structs). Named fields are removed.
Map types use guillemets: `Name.<<K V>>`.

### Kind declarations

Simple kind: `Name.[ capabilities ]`.
Complex kind: `Name.{ [superkinds] [associated-types] <<constants>> [capabilities] }`.
Capability receivers: `.` shared, `!` mutable, `:` no self.
Associated constants are upper case in the guillemet delimiter.

### Signal wire

The contract id (Channel) is dropped; one contract per socket. The Signal's
version is the wire version. The generated wire types are:

- `Version(u16, u16, u16)` — rkyv tuple struct
- `Refusal.[ VersionMismatch.{ Version Version } Unreadable ]`
- `Body.[ Request.Request Reply.Reply Refusal.Refusal ]`
- `Frame.{ Version Body }` — rkyv tuple struct

All declared types and the wire envelope carry rkyv derives.

### CLI

The `ethos-zero` binary is a direct datom tool, not a Nexus client:

```
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
```

Prints `Generated.[ /abs/out-dir/file.rs ]` on success,
`GenerationFailure.{ path reason }` on failure.
With no argument, prints its own ethos (self-description).

The Nexus runtime (`ethos-zero-nexus`), both edge CLIs (`ethos-zero`,
`ethos-zero-meta`), and signal-ethos-zero are removed. Consumers that ran
the Nexus should call the CLI directly or use the library API.

### Dependencies

Protos moves to the ProtoformStack branch (0.15.0). Datomic moves to its
ProtoformStack branch. Pin the revisions in `Cargo.toml` and the
corresponding flake inputs.

### Migration

1. Replace `FileReader::new(&manifest).read(source)` with `ethos_zero::read(source)`.
2. Replace `RustEmitter::new().emit(&file)` with `ethos_zero::emit(&concept)`.
3. Rewrite `.ethos` files from Interface/Schema to Signal/Library syntax.
4. Remove Channel declarations, Visibility labels, and named field syntax.
5. For signal crates: remove the `signal-frame` dependency; the wire types are
   generated directly with rkyv derives.
6. Remove the `ethos-zero-nexus` service and its socket configuration.
