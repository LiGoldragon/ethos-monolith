# Upgrades

## 0.1.0 — ethos-zero E0–E2 replacement

The former `ethos-monolith` package and repository are renamed `ethos-zero`.
The legacy text parser, build/generation APIs, fixtures, and architecture
guards are removed. Consumers must use the new `FileReader` and `RustEmitter`
APIs and pin Protos 0.14 / Datomic 0.7 at the revisions in `Cargo.toml`.
