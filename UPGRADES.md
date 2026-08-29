# Upgrades

## Unreleased

WireContract 0.3.1 makes generated `String` aliases invariant-bearing: their
fields are private and construction validates Datomic representability through
`TryFrom`. Unsupported tuple-struct declarations now return a typed
`FileFault` instead of emitting an empty Datomic implementation.

E2 now checks the complete map-owned public declaration contract for Protos
and Datomic through syntax projection. The map pins advance to the complete
contract revisions. This does not change Ethos-zero's public runtime API.

## 0.1.0 — ethos-zero E0–E2 replacement

The former `ethos-monolith` package and repository are renamed `ethos-zero`.
The legacy text parser, build/generation APIs, fixtures, and architecture
guards are removed. Consumers must use the new `FileReader` and `RustEmitter`
APIs and pin Protos 0.14 / Datomic 0.7 at the revisions in `Cargo.toml`.
