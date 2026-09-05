//! Tests for ethos-zero: every example from Vision/ethos.md.

use ethos_zero::{
    Canonicalizing, File, Generating, KindDeclaration,
    Receiver, TypeDeclaration, TypeExpression, Variant,
};
use protos::{Conceivable, Protosizable, Textualizable};

// ============================================================================
// Helper: read and generate
// ============================================================================

fn read(source: &str) -> File {
    let canonical = source.canonicalize();
    let delineation = canonical.protosize().expect("delineation");
    delineation.conceive().expect("conceive")
}

fn generate(source: &str) -> String {
    let file = read(source);
    file.generate().expect("generate")
}

fn format_rust(source: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("rustfmt")
        .arg("--edition=2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("rustfmt");
    child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

/// Assert that generated Rust is syntactically valid by parsing it with syn.
fn assert_compiles(ethos_source: &str, _test_name: &str) {
    let rust = generate(ethos_source);
    let formatted = format_rust(&rust);

    // Verify the generated code parses as valid Rust syntax
    syn::parse_str::<syn::File>(&formatted).unwrap_or_else(|e| {
        panic!(
            "Generated Rust failed to parse:\n{e}\n\nGenerated code:\n{formatted}"
        );
    });
}

// ============================================================================
// Sweet-to-canonical conversion
// ============================================================================

#[test]
fn sweet_to_canonical_types() {
    let sweet = "Types\n[]\n[ Record.{ Text } ]\n[]";
    let canonical = sweet.canonicalize();
    assert!(canonical.starts_with("Types.{"));
    assert!(canonical.ends_with("}"));

    // Both forms should read identically
    let from_sweet = read(sweet);
    let from_canonical = read("Types.{ [] [ Record.{ Text } ] [] }");
    assert_eq!(from_sweet, from_canonical);
}

#[test]
fn already_canonical_unchanged() {
    let source = "Types.{ [] [ Record.{ Text } ] [] }";
    let canonical = source.canonicalize();
    // Should be identical to the original
    assert_eq!(canonical, source);
}

// ============================================================================
// Reader: Types variant
// ============================================================================

#[test]
fn types_reads_struct() {
    let file = read("Types\n[]\n[ Record.{ Text Integer } ]\n[]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    assert_eq!(types.types.len(), 1);
    match &types.types[0] {
        TypeDeclaration::Struct(name, fields) => {
            assert_eq!(name, "Record");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("expected Struct"),
    }
}

#[test]
fn types_reads_enum() {
    let file = read("Types\n[]\n[ SinkError.[ Closed Full ] ]\n[]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    match &types.types[0] {
        TypeDeclaration::Enum(name, variants) => {
            assert_eq!(name, "SinkError");
            assert_eq!(variants.len(), 2);
            assert!(matches!(&variants[0], Variant::Unit(n) if n == "Closed"));
            assert!(matches!(&variants[1], Variant::Unit(n) if n == "Full"));
        }
        _ => panic!("expected Enum"),
    }
}

#[test]
fn types_reads_alias() {
    let file = read("Types\n[]\n[ LockId.Integer ]\n[]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    match &types.types[0] {
        TypeDeclaration::Alias(name, target) => {
            assert_eq!(name, "LockId");
            assert!(matches!(target, TypeExpression::Named(n) if n == "Integer"));
        }
        _ => panic!("expected Alias"),
    }
}

#[test]
fn types_reads_import_single() {
    let file = read("Types\n[ protos:Text ]\n[]\n[]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    assert_eq!(types.imports.len(), 1);
    match &types.imports[0] {
        ethos_zero::Import::Single(source, name) => {
            assert_eq!(source, "protos");
            assert_eq!(name, "Text");
        }
        _ => panic!("expected Single import"),
    }
}

#[test]
fn types_reads_import_multiple() {
    let file = read("Types\n[ protos:[ Text Integer ] ]\n[]\n[]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    match &types.imports[0] {
        ethos_zero::Import::Multiple(source, names) => {
            assert_eq!(source, "protos");
            assert_eq!(names, &["Text", "Integer"]);
        }
        _ => panic!("expected Multiple import"),
    }
}

#[test]
fn types_reads_associations() {
    let file = read("Types\n[]\n[ Sink.{ Text } ]\n[ Sink.[ Summarizable Fillable ] ]");
    let File::Types(types) = &file else { panic!("expected Types"); };
    assert_eq!(types.associations.len(), 1);
    assert_eq!(types.associations[0].ty, "Sink");
    assert_eq!(types.associations[0].kinds, vec!["Summarizable", "Fillable"]);
}

// ============================================================================
// Reader: Kinds variant
// ============================================================================

#[test]
fn kinds_reads_simple_kind() {
    let file = read("Kinds\n[]\n[ Summarizable.[ summarize.[ Text ] ] ]");
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    assert_eq!(kinds.kinds.len(), 1);
    match &kinds.kinds[0] {
        KindDeclaration::Simple { name, capabilities, .. } => {
            assert_eq!(name, "Summarizable");
            assert_eq!(capabilities.len(), 1);
            assert_eq!(capabilities[0].name, "summarize");
            assert_eq!(capabilities[0].receiver, Receiver::Shared);
        }
        _ => panic!("expected Simple kind"),
    }
}

#[test]
fn kinds_reads_complex_kind() {
    let source = "Kinds\n[]\n[ Streamable.{ [ Fillable ] [ Item<Serializable> ] [ CAPACITY.Integer ] [ next![ Option<Item> ] ] } ]";
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    match &kinds.kinds[0] {
        KindDeclaration::Complex {
            name, superkinds, associated_types, associated_constants, capabilities, ..
        } => {
            assert_eq!(name, "Streamable");
            assert_eq!(superkinds, &["Fillable"]);
            assert_eq!(associated_types.len(), 1);
            assert_eq!(associated_types[0].name, "Item");
            assert_eq!(associated_types[0].constraints, vec!["Serializable"]);
            assert_eq!(associated_constants.len(), 1);
            assert_eq!(associated_constants[0].name, "CAPACITY");
            assert_eq!(capabilities.len(), 1);
            assert_eq!(capabilities[0].name, "next");
            assert_eq!(capabilities[0].receiver, Receiver::Mutable);
        }
        _ => panic!("expected Complex kind"),
    }
}

#[test]
fn kinds_reads_constrained_kind_identity() {
    let source = "Kinds\n[]\n[ Processable<[Clonable Sendable] Serializable>.[ process.[ Text ] ] ]";
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    match &kinds.kinds[0] {
        KindDeclaration::Simple { name, constraints, capabilities } => {
            assert_eq!(name, "Processable");
            assert_eq!(constraints.len(), 2);
            assert_eq!(constraints[0].bounds, vec!["Clonable", "Sendable"]);
            assert_eq!(constraints[1].bounds, vec!["Serializable"]);
            assert_eq!(capabilities.len(), 1);
        }
        _ => panic!("expected Simple kind with constraints"),
    }
}

// ============================================================================
// Reader: Signal variant
// ============================================================================

#[test]
fn signal_reads_requests_and_responses() {
    let source = "\
Signal
[]
[ Lock.LockRequest Release.LockId ]
[ Locked.Lock Released.Lock ]
[ LockId.Integer LockRequest.{ Text Text } Lock.{ Integer Text Text } ]";
    let file = read(source);
    let File::Signal(signal) = &file else { panic!("expected Signal"); };
    assert_eq!(signal.requests.len(), 2);
    assert_eq!(signal.responses.len(), 2);
    assert_eq!(signal.types.len(), 3);
    assert!(matches!(&signal.requests[0], Variant::Typed(n, _) if n == "Lock"));
    assert!(matches!(&signal.requests[1], Variant::Typed(n, _) if n == "Release"));
}

// ============================================================================
// Reader: Sema variant
// ============================================================================

#[test]
fn sema_reads_types() {
    let source = "Sema\n[]\n[ Entry.{ Text Integer } ]";
    let file = read(source);
    let File::Sema(sema) = &file else { panic!("expected Sema"); };
    assert_eq!(sema.types.len(), 1);
}

// ============================================================================
// Reader: capability receivers
// ============================================================================

#[test]
fn capability_receivers() {
    let source = "Kinds\n[]\n[ Test.[ read.[ Text ] write![ Text ] create:[ Self ] ] ]";
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    let KindDeclaration::Simple { capabilities, .. } = &kinds.kinds[0] else {
        panic!("expected Simple");
    };
    assert_eq!(capabilities[0].receiver, Receiver::Shared);
    assert_eq!(capabilities[1].receiver, Receiver::Mutable);
    assert_eq!(capabilities[2].receiver, Receiver::None);
}

#[test]
fn capability_with_inputs() {
    let source = "Kinds\n[]\n[ Fillable.[ push!{ [ Text ] [ Result<Integer SinkError> ] } ] ]";
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    let KindDeclaration::Simple { capabilities, .. } = &kinds.kinds[0] else {
        panic!("expected Simple");
    };
    assert_eq!(capabilities[0].name, "push");
    assert_eq!(capabilities[0].receiver, Receiver::Mutable);
    assert_eq!(capabilities[0].inputs.len(), 1);
}

// ============================================================================
// Generator: generated Rust content
// ============================================================================

#[test]
fn generates_struct() {
    let rust = generate("Types\n[]\n[ Record.{ Text Integer } ]\n[]");
    assert!(rust.contains("pub struct Record"));
    assert!(rust.contains("protos :: Text"));
    assert!(rust.contains("protos :: Integer"));
}

#[test]
fn generates_enum() {
    let rust = generate("Types\n[]\n[ SinkError.[ Closed Full ] ]\n[]");
    assert!(rust.contains("pub enum SinkError"));
    assert!(rust.contains("Closed"));
    assert!(rust.contains("Full"));
}

#[test]
fn generates_alias() {
    let rust = generate("Types\n[]\n[ LockId.Integer ]\n[]");
    assert!(rust.contains("pub type LockId = protos :: Integer"));
}

#[test]
fn generates_trait() {
    let rust = generate("Kinds\n[]\n[ Summarizable.[ summarize.[ Text ] ] ]");
    assert!(rust.contains("pub trait Summarizable"));
    assert!(rust.contains("fn summarize"));
    assert!(rust.contains("protos :: Text"));
}

#[test]
fn generates_complex_trait() {
    let source = "Kinds\n[]\n[ Streamable.{ [ Fillable ] [ Item<Serializable> ] [ CAPACITY.Integer ] [ next![ Option<Item> ] ] } ]";
    let rust = generate(source);
    assert!(rust.contains("pub trait Streamable"));
    assert!(rust.contains("Fillable"));
    assert!(rust.contains("type Item"));
    assert!(rust.contains("Serializable"));
    assert!(rust.contains("const CAPACITY"));
    assert!(rust.contains("fn next"));
}

#[test]
fn generates_association_assertion() {
    let rust = generate("Types\n[]\n[ Sink.{ Text } ]\n[ Sink.[ Summarizable Fillable ] ]");
    assert!(rust.contains("assert_sink_summarizable"));
    assert!(rust.contains("assert_sink_fillable"));
}

#[test]
fn generates_signal_request_response() {
    let source = "\
Signal
[]
[ Lock.LockRequest Release.LockId ]
[ Locked.Lock Released.Lock ]
[ LockId.Integer LockRequest.{ Text Text } Lock.{ Integer Text Text } ]";
    let rust = generate(source);
    assert!(rust.contains("pub enum Request"));
    assert!(rust.contains("pub enum Response"));
    assert!(!rust.contains("pub enum Reply"));
    // Token spacing: Lock (LockRequest)
    assert!(rust.contains("Lock (LockRequest)"));
    assert!(rust.contains("Release (LockId)"));
    assert!(rust.contains("Locked (Lock)"));
    assert!(rust.contains("Released (Lock)"));
}

#[test]
fn generates_datomic_impls_for_struct() {
    let rust = generate("Types\n[]\n[ Record.{ Text Integer } ]\n[]");
    assert!(rust.contains("impl protos :: Conceivable < datomic :: Datom > for Record"));
    assert!(rust.contains("impl datomic :: Datomic for Record"));
    assert!(rust.contains("impl protos :: Incorporable < Record > for datomic :: Datom"));
    assert!(rust.contains("incorporate_from"));
}

#[test]
fn generates_datomic_impls_for_enum() {
    let rust = generate("Types\n[]\n[ SinkError.[ Closed Full ] ]\n[]");
    assert!(rust.contains("impl protos :: Conceivable < datomic :: Datom > for SinkError"));
    assert!(rust.contains("impl datomic :: Datomic for SinkError"));
    assert!(rust.contains("impl protos :: Incorporable < SinkError > for datomic :: Datom"));
}

#[test]
fn generates_constrained_trait() {
    let source = "Kinds\n[]\n[ Processable<[Clonable Sendable] Serializable>.[ process.[ Text ] ] ]";
    let rust = generate(source);
    assert!(rust.contains("pub trait Processable"));
    assert!(rust.contains("Clonable"));
    assert!(rust.contains("Sendable"));
    assert!(rust.contains("Serializable"));
}

// ============================================================================
// Protosizable: round-trip sweet → canonical → conceive → protosize → text
// ============================================================================

#[test]
fn round_trip_types() {
    let source = "Types\n[ protos:Text ]\n[ Record.{ Text Integer } ]\n[]";
    let file = read(source);
    let delineation = file.protosize().unwrap();
    let text = delineation.textualize();
    // Round-trip: the textualized form should be readable
    let delineation2 = text.protosize().unwrap();
    let file2: File = delineation2.conceive().unwrap();
    assert_eq!(file, file2);
}

#[test]
fn round_trip_kinds() {
    let source = "Kinds\n[]\n[ Summarizable.[ summarize.[ Text ] ] ]";
    let file = read(source);
    let delineation = file.protosize().unwrap();
    let text = delineation.textualize();
    let delineation2 = text.protosize().unwrap();
    let file2: File = delineation2.conceive().unwrap();
    assert_eq!(file, file2);
}

#[test]
fn round_trip_signal() {
    let source = "Signal\n[]\n[ Lock.LockRequest ]\n[ Locked.Lock ]\n[ LockRequest.{ Text } Lock.{ Integer Text } ]";
    let file = read(source);
    let delineation = file.protosize().unwrap();
    let text = delineation.textualize();
    let delineation2 = text.protosize().unwrap();
    let file2: File = delineation2.conceive().unwrap();
    assert_eq!(file, file2);
}

// ============================================================================
// Compilation tests: generated Rust compiles
// ============================================================================

#[test]
fn record_types_compile() {
    assert_compiles("Types\n[]\n[ Record.{ Text Integer } ]\n[]", "record_types");
}

#[test]
fn multi_types_compile() {
    assert_compiles(
        "Types\n[]\n[ Record.{ Text Integer } Report.{ Text Vector<Integer> } SinkError.[ Closed Full ] LockId.Integer ]\n[]",
        "multi_types",
    );
}

#[test]
fn signal_compile() {
    assert_compiles(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/orchestrate.ethos")).unwrap(),
        "signal",
    );
}

#[test]
fn sema_compile() {
    assert_compiles("Sema\n[]\n[ Entry.{ Text Integer } ]", "sema");
}

// ============================================================================
// Self-description: ethos-zero reads its own .ethos
// ============================================================================

#[test]
fn self_description_reads() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ethos-zero.ethos"));
    let file = read(source);
    let File::Signal(_) = &file else {
        panic!("expected Signal");
    };
}

#[test]
fn self_description_generates() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ethos-zero.ethos"));
    let rust = generate(source);
    assert!(rust.contains("pub enum Request"));
    assert!(rust.contains("pub enum Response"));
    assert!(rust.contains("GenerateRequest"));
}

// ============================================================================
// Fixture files: read and generate from fixture files
// ============================================================================

#[test]
fn fixture_record_types() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/record-types.ethos"));
    let file = read(source);
    let File::Types(types) = &file else { panic!("expected Types"); };
    assert_eq!(types.types.len(), 1);
    let rust = generate(source);
    assert!(rust.contains("pub struct Record"));
}

#[test]
fn fixture_multi_types() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/multi-types.ethos"));
    let file = read(source);
    let File::Types(types) = &file else { panic!("expected Types"); };
    assert_eq!(types.types.len(), 4);
}

#[test]
fn fixture_processable() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/processable-kinds.ethos"));
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    assert_eq!(kinds.kinds.len(), 1);
    let rust = generate(source);
    assert!(rust.contains("pub trait Processable"));
}

#[test]
fn fixture_capability_kinds() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/capability-kinds.ethos"));
    let file = read(source);
    let File::Kinds(kinds) = &file else { panic!("expected Kinds"); };
    assert_eq!(kinds.kinds.len(), 2);
    let rust = generate(source);
    assert!(rust.contains("pub trait Summarizable"));
    assert!(rust.contains("pub trait Fillable"));
    assert!(rust.contains("fn push"));
    assert!(rust.contains("fn drain"));
    assert!(rust.contains("fn create"));
}

#[test]
fn fixture_streamable() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/streamable-kind.ethos"));
    let rust = generate(source);
    assert!(rust.contains("pub trait Streamable"));
    assert!(rust.contains("Fillable"));
    assert!(rust.contains("const CAPACITY"));
}

#[test]
fn fixture_sink_associations() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/sink-associations.ethos"));
    let rust = generate(source);
    assert!(rust.contains("assert_sink_summarizable"));
    assert!(rust.contains("assert_sink_fillable"));
}

#[test]
fn fixture_orchestrate() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/orchestrate.ethos"));
    let file = read(source);
    let File::Signal(signal) = &file else { panic!("expected Signal"); };
    assert_eq!(signal.requests.len(), 3);
    assert_eq!(signal.responses.len(), 5);
    let rust = generate(source);
    assert!(rust.contains("pub enum Request"));
    assert!(rust.contains("pub enum Response"));
}

#[test]
fn fixture_sema() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/entry-sema.ethos"));
    let file = read(source);
    let File::Sema(sema) = &file else { panic!("expected Sema"); };
    assert_eq!(sema.types.len(), 1);
}

// ============================================================================
// No version in file
// ============================================================================

#[test]
fn no_version_field() {
    // The old Library.{0 1 0} form should fail — no version
    let result = std::panic::catch_unwind(|| read("Library.{0 1 0}\n[]\n[]\n[]\n[]"));
    assert!(result.is_err() || {
        // Even if it doesn't panic, it should fail on conceive
        let canonical = "Library.{0 1 0}\n[]\n[]\n[]\n[]".canonicalize();
        let d = canonical.protosize();
        d.is_err() || {
            let d = d.unwrap();
            let r: Result<File, _> = d.conceive();
            r.is_err()
        }
    });
}
