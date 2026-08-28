# Architecture

```text
Ethos Text -> Protos delineation -> Portion -> FileReader -> File
File -> RustEmitter -> quote tokens -> syn::File -> Rust text
```

Ethos-zero has no character parser. `FileReader` only asks structural
questions of Protos portions. `RustEmitter` constructs syntax nodes and
validates them as `syn::File`; it does not assemble Rust source strings.

An external `ImportReference` receives a `FileLocation` solely from the
caller-provided manifest. A local `file.[…]` import is explicitly local.
