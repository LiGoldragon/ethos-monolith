# ethos-zero

The ethos schema language, version zero. Reads ethos, emits Rust.

Text arrives as sweet form, is mechanically canonicalized, delineated
through protos, conceived into the File declaration model, and
generated into Rust through the Generating kind.

## File variants

- **Types**: imports, type declarations, associations
- **Kinds**: imports, kind (trait) declarations
- **Signal**: imports, request variants, response variants, types
- **Sema**: imports, storage/record types (implied Datomic)

## CLI

```
ethos-zero 'Generate.{ /abs/file.ethos /abs/out-dir }'
# -> Generated.[ /abs/out-dir/file.rs ]
```

With no argument, prints its own ethos.
