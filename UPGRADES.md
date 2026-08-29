# Upgrades

## 0.7.0 — data-only Datomic libraries

`RustEmitter::datomic_library()` emits a Schema's public declarations and
executable Datomic Portion anatomy while intentionally omitting Schema kinds
and associations. It is for committed typed data libraries, not Signal
contracts: it creates no Interface roots, frames, envelopes, channel metadata,
or rkyv projection.

## 0.6.0 — named record fields

Ethos record declarations now accept `Name.Type` field portions. The name is
the generated Rust field name; the type is the field's Datomic anatomy; and
the written declaration order remains the positional Datomic record order.
Use this form whenever a record repeats a type or needs a semantic name that
does not equal its type. Other head separators are rejected, so a source must
use the ruled period form rather than relying on an accidental parse.

## 0.7.2 — ordered Nexus subscription activation

Nexus now sequences each subscription's initial `Observed` reply before any
subsequent stream event for that connection. Registration still happens before
the snapshot is read, but an activation barrier holds actor-delivered changes
until the socket writer has emitted the snapshot. This preserves the ordinary
and meta wire formats while restoring the state-then-changes contract.

## 0.7.1 — Nexus socket and containment completion

Subscription connections now retain a private runtime session for their full
Unix-socket lifetime: Subscribe, event delivery, and Unsubscribe occur on the
same connection, foreign clients cannot remove another client's subscription,
and disconnect removes all of that connection's registrations. On reopening a
pre-0.7 store, Nexus atomically restores both durable socket paths to the
currently bound XDG-derived paths while preserving the source manifest.

Generation now traverses source roots through directory descriptors with
`O_NOFOLLOW`, creates destination parents through those descriptors, and
publishes a synced temporary artifact with `renameat`. Existing and
prospective symlink paths are rejected with `InvalidRelativePath`; deploy the
0.7.1 runtime before relying on generation in a shared source root.

## 0.7.0 — audited Nexus runtime boundary correction

The Nexus and both CLIs now share the same XDG fallback socket directory and
reject frames over 16 MiB before allocation. Source and artifact paths are
canonical containment-checked, including symlink traversal. Configure now
rejects changed socket paths until listener replacement can be atomic; it may
still persist a changed source-manifest path. Deploy this workspace update as a
unit: stop the old `ethos-zero-nexus`, then start the new executable so its
socket location and persisted configuration are interpreted consistently.

## 0.6.0 — Ethos-zero Nexus runtime

Ethos-zero is now shipped as a Nexus workspace: `ethos-zero-nexus` owns the
durable Sema state and serves generated ordinary and meta WireContract frames;
`ethos-zero` and `ethos-zero-meta` are the corresponding one-Datom CLIs.  The
runtime persists its default configuration on first open, uses the XDG state
and runtime locations, and rejects source paths that escape their configured
source root with the typed `InvalidRelativePath` refusal.

## Unreleased

WireContract 0.5.0 makes every generated interface root (request, reply,
refusal, and stream event) a Datomic anatomy as well as an rkyv projection.
This enables source-linked textual boundary tests without a hand-maintained
root codec.

WireContract 0.4.0 frames can now carry generated refusal roots and stream
events as `FrameBody::Refusal` and `FrameBody::Event`. Consumers that decode
frames exhaustively must handle these additional wire variants.

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
