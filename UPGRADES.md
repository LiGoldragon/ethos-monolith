# Upgrades

## 0.5.0 — generated Datom projections

Generated Signal modules now implement typed Datom realization and
textualization instead of legacy Dotos codecs. Deploy this producer before any
consumer: update the generator revision, add the matching Datom dependency to
each signal crate, regenerate `signal.rs`, and replace Dotos text fixtures
with Datom round trips against the generated request root. This is a clean
break: Dotos traits, parser entry points, and old command fixtures are absent
from the regenerated module.

## 0.4.0 — signal-only wire generation

This release removes the three-artifact `ComponentGeneration` API and the
non-signal Interface projection. Wire consumers must replace it with
`SignalGeneration`, retain only `signal.ethos`, and regenerate only
`signal.rs`. Every signal source must carry its `Channel.{Name ContractId
WireRevision}` declaration immediately after the Interface header; a missing
Channel now rejects generation.

Deploy by updating the producer to 0.4.0 before updating a consumer. In each
consumer, remove `nexus.ethos`, `sema.ethos`, `nexus.rs`, and `sema.rs` from
the wire-generation step and manifests, then commit the regenerated
`signal.rs`. No compatibility API exists.
