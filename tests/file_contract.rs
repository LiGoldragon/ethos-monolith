use ethos_zero::{Actualizing, Concept, Emitting, Potential, Version};
use protos::{Printing, Protosizable};
use std::fs;

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Reader tests
// ---------------------------------------------------------------------------

#[test]
fn library_sweet_form_reads_version_and_sections() {
    let source = "\
Library.{0 1 0}
[]
[ Sink.{ Text Vector<Text> }
  SinkError.[ Closed Full ] ]
[]
[]";
    let concept = Potential::from(source)
        .actualize()
        .expect("library sweet form");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.version, Version(0, 1, 0));
    assert!(library.imports.is_empty());
    assert_eq!(library.types.len(), 2);
    assert!(library.kinds.is_empty());
    assert!(library.associations.is_empty());
}

#[test]
fn library_full_form_reads_identically() {
    let source = "Library.{ {0 1 0} [] [ Sink.{ Text } ] [] [] }";
    let concept = Potential::from(source)
        .actualize()
        .expect("library full form");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.version, Version(0, 1, 0));
    assert_eq!(library.types.len(), 1);
}

#[test]
fn signal_sweet_form_reads_requests_and_responses() {
    let source = "\
Signal.{1 0 0}
[]
[ Lock.LockRequest Release.LockId ]
[ Locked.Lock Released.Lock ]
[ LockId.Integer LockRequest.{ Text Text } Lock.{ Integer Text Text } ]";
    let concept = Potential::from(source)
        .actualize()
        .expect("signal sweet form");
    let Concept::Signal(signal) = &concept else {
        panic!("expected Signal");
    };
    assert_eq!(signal.version, Version(1, 0, 0));
    assert_eq!(signal.requests.len(), 2);
    assert_eq!(signal.responses.len(), 2);
    assert_eq!(signal.types.len(), 3);
}

#[test]
fn single_import_reads_source_and_name() {
    let source = "Library.{0 1 0} [protos:Text] [] [] []";
    let concept = Potential::from(source).actualize().expect("import");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.imports.len(), 1);
    match &library.imports[0] {
        ethos_zero::Import::Single { source, name } => {
            assert_eq!(source, "protos");
            assert_eq!(name, "Text");
        }
        _ => panic!("expected Single import"),
    }
}

#[test]
fn multiple_import_reads_source_and_names() {
    let source = "Library.{0 1 0} [protos:[ Text Integer ]] [] [] []";
    let concept = Potential::from(source).actualize().expect("import");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.imports.len(), 1);
    match &library.imports[0] {
        ethos_zero::Import::Multiple { source, names } => {
            assert_eq!(source, "protos");
            assert_eq!(names, &["Text", "Integer"]);
        }
        _ => panic!("expected Multiple import"),
    }
}

#[test]
fn struct_declaration_reads_positional_fields() {
    let source = "Library.{0 1 0} [] [ Pair.{ Text Integer } ] [] []";
    let concept = Potential::from(source).actualize().expect("struct");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Struct { name, fields } => {
            assert_eq!(name, "Pair");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("expected Struct"),
    }
}

#[test]
fn enum_declaration_reads_unit_and_typed_variants() {
    let source = "Library.{0 1 0} [] [ Error.[ Closed Full NotFound.Text ] ] [] []";
    let concept = Potential::from(source).actualize().expect("enum");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Enum { name, variants } => {
            assert_eq!(name, "Error");
            assert_eq!(variants.len(), 3);
            assert!(matches!(&variants[0], ethos_zero::Variant::Unit(n) if n == "Closed"));
            assert!(matches!(&variants[1], ethos_zero::Variant::Unit(n) if n == "Full"));
            assert!(matches!(&variants[2], ethos_zero::Variant::Typed(n, _) if n == "NotFound"));
        }
        _ => panic!("expected Enum"),
    }
}

#[test]
fn alias_declaration_reads_target_type() {
    let source = "Library.{0 1 0} [] [ LockId.Integer ] [] []";
    let concept = Potential::from(source).actualize().expect("alias");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Alias { name, target } => {
            assert_eq!(name, "LockId");
            assert!(matches!(target, ethos_zero::TypeExpression::Named(n) if n == "Integer"));
        }
        _ => panic!("expected Alias"),
    }
}

#[test]
fn applied_type_expression_reads_constructor_and_arguments() {
    let source = "Library.{0 1 0} [] [ Items.Vector<Text> ] [] []";
    let concept = Potential::from(source).actualize().expect("applied type");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Alias { name, target } => {
            assert_eq!(name, "Items");
            match target {
                ethos_zero::TypeExpression::Applied {
                    constructor,
                    arguments,
                } => {
                    assert_eq!(constructor, "Vector");
                    assert_eq!(arguments.len(), 1);
                }
                _ => panic!("expected Applied"),
            }
        }
        _ => panic!("expected Alias"),
    }
}

#[test]
fn map_declaration_reads_key_and_value_types() {
    let source = "Library.{0 1 0} [] [ Roles.\u{00AB}Text Integer\u{00BB} ] [] []";
    let concept = Potential::from(source).actualize().expect("map");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Map { name, key, value } => {
            assert_eq!(name, "Roles");
            assert!(matches!(key, ethos_zero::TypeExpression::Named(n) if n == "Text"));
            assert!(matches!(value, ethos_zero::TypeExpression::Named(n) if n == "Integer"));
        }
        _ => panic!("expected Map"),
    }
}

#[test]
fn simple_kind_reads_capabilities() {
    let source = "Library.{0 1 0} [] [] [ Summarizable.[ summarize.[ Text ] ] ] []";
    let concept = Potential::from(source).actualize().expect("simple kind");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.kinds.len(), 1);
    match &library.kinds[0] {
        ethos_zero::KindDeclaration::Simple {
            name, capabilities, ..
        } => {
            assert_eq!(name, "Summarizable");
            assert_eq!(capabilities.len(), 1);
            assert_eq!(capabilities[0].name, "summarize");
            assert_eq!(capabilities[0].receiver, ethos_zero::Receiver::Shared);
        }
        _ => panic!("expected Simple kind"),
    }
}

#[test]
fn capability_with_inputs_reads_receiver_and_types() {
    let source = "Library.{0 1 0} [] [] [ Fillable.[ push!{ [ Text ] [ Integer ] } ] ] []";
    let concept = Potential::from(source)
        .actualize()
        .expect("capability with inputs");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.kinds[0] {
        ethos_zero::KindDeclaration::Simple { capabilities, .. } => {
            assert_eq!(capabilities[0].name, "push");
            assert_eq!(capabilities[0].receiver, ethos_zero::Receiver::Mutable);
            assert_eq!(capabilities[0].inputs.len(), 1);
        }
        _ => panic!("expected Simple kind"),
    }
}

#[test]
fn static_capability_has_no_self() {
    let source = "Library.{0 1 0} [] [] [ Factory.[ create:[ Self ] ] ] []";
    let concept = Potential::from(source)
        .actualize()
        .expect("static capability");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.kinds[0] {
        ethos_zero::KindDeclaration::Simple { capabilities, .. } => {
            assert_eq!(capabilities[0].name, "create");
            assert_eq!(capabilities[0].receiver, ethos_zero::Receiver::None);
            assert!(matches!(
                &capabilities[0].yield_type,
                ethos_zero::TypeExpression::SelfType
            ));
        }
        _ => panic!("expected Simple kind"),
    }
}

#[test]
fn association_reads_type_and_kinds() {
    let source = "Library.{0 1 0} [] [] [] [ Sink.[ Summarizable Fillable ] ]";
    let concept = Potential::from(source).actualize().expect("association");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    assert_eq!(library.associations.len(), 1);
    assert_eq!(library.associations[0].ty, "Sink");
    assert_eq!(library.associations[0].kinds, ["Summarizable", "Fillable"]);
}

#[test]
fn inline_struct_variant_reads_fields() {
    let source = "Library.{0 1 0} [] [ P.[ Headed.{ Text Integer } Bare ] ] [] []";
    let concept = Potential::from(source)
        .actualize()
        .expect("inline struct variant");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Enum { variants, .. } => {
            assert!(
                matches!(&variants[0], ethos_zero::Variant::InlineStruct(n, fields) if n == "Headed" && fields.len() == 2)
            );
            assert!(matches!(&variants[1], ethos_zero::Variant::Unit(n) if n == "Bare"));
        }
        _ => panic!("expected Enum"),
    }
}

#[test]
fn inline_enum_variant_reads_inner_variants() {
    let source = "Library.{0 1 0} [] [ Outer.[ Inner.[ A B ] ] ] [] []";
    let concept = Potential::from(source)
        .actualize()
        .expect("inline enum variant");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.types[0] {
        ethos_zero::TypeDeclaration::Enum { variants, .. } => match &variants[0] {
            ethos_zero::Variant::InlineEnum(name, inner) => {
                assert_eq!(name, "Inner");
                assert_eq!(inner.len(), 2);
            }
            _ => panic!("expected InlineEnum"),
        },
        _ => panic!("expected Enum"),
    }
}

#[test]
fn invalid_root_faults() {
    assert!(
        Potential::from("Unknown.{0 1 0} [] [] [] []")
            .actualize()
            .is_err()
    );
}

#[test]
fn empty_input_faults() {
    assert!(Potential::from("").actualize().is_err());
}

// ---------------------------------------------------------------------------
// Emitter tests
// ---------------------------------------------------------------------------

#[test]
fn library_emits_parseable_rust() {
    let source = "\
Library.{0 1 0}
[]
[ Sink.{ Text Vector<Text> }
  SinkError.[ Closed Full ] ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    syn::parse_file(&rust).expect("generated Rust parses");
}

#[test]
fn signal_emits_parseable_rust() {
    let source = "\
Signal.{1 0 0}
[]
[ Lock.LockRequest ]
[ Locked.Lock ]
[ LockRequest.{ Text Text } Lock.{ Integer Text Text } ]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    syn::parse_file(&rust).expect("generated Rust parses");
}

#[test]
fn alias_emits_type_alias() {
    let source = "Library.{0 1 0} [] [ LockId.Integer ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(rust.contains("type LockId"));
}

#[test]
fn struct_emits_tuple_struct() {
    let source = "Library.{0 1 0} [] [ Pair.{ Text Integer } ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    // Verify it parses
    let syntax = syn::parse_file(&rust).expect("generated Rust parses");
    // Find the struct
    let has_pair = syntax
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(s) if s.ident == "Pair"));
    assert!(has_pair, "expected Pair struct in: {rust}");
}

#[test]
fn enum_emits_variants() {
    let source = "Library.{0 1 0} [] [ Color.[ Red Green Blue ] ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    let syntax = syn::parse_file(&rust).expect("generated Rust parses");
    let has_enum = syntax
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Enum(e) if e.ident == "Color"));
    assert!(has_enum, "expected Color enum");
}

#[test]
fn kind_emits_trait() {
    let source = "Library.{0 1 0} [] [] [ Printable.[ print.[ Text ] ] ] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    let syntax = syn::parse_file(&rust).expect("generated Rust parses");
    let has_trait = syntax
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Trait(t) if t.ident == "Printable"));
    assert!(has_trait, "expected Printable trait");
}

#[test]
fn datomic_impl_is_generated_for_struct() {
    let source = "Library.{0 1 0} [] [ Pair.{ Text Integer } ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(rust.contains("impl datomic :: Datomic for Pair"));
}

#[test]
fn datomic_impl_is_generated_for_enum() {
    let source = "Library.{0 1 0} [] [ Color.[ Red Green Blue ] ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(rust.contains("impl datomic :: Datomic for Color"));
}

// ---------------------------------------------------------------------------
// Fixture tests
// ---------------------------------------------------------------------------

#[test]
fn fixture_library_reads_and_emits() {
    let source = fs::read_to_string("fixtures/example-library.ethos").expect("fixture library");
    let concept = Potential::from(source.as_str())
        .actualize()
        .expect("read fixture library");
    assert!(matches!(&concept, Concept::Library(_)));
    let rust = concept.emit().expect("emit fixture library");
    syn::parse_file(&rust).expect("fixture library Rust parses");
}

#[test]
fn fixture_signal_reads_and_emits() {
    let source = fs::read_to_string("fixtures/orchestrate.ethos").expect("fixture signal");
    let concept = Potential::from(source.as_str())
        .actualize()
        .expect("read fixture signal");
    assert!(matches!(&concept, Concept::Signal(_)));
    let rust = concept.emit().expect("emit fixture signal");
    syn::parse_file(&rust).expect("fixture signal Rust parses");
}

#[test]
fn self_description_reads() {
    let source = fs::read_to_string("ethos-zero.ethos").expect("self-description");
    let concept = Potential::from(source.as_str())
        .actualize()
        .expect("read self-description");
    assert!(matches!(&concept, Concept::Library(_)));
    let rust = concept.emit().expect("emit self-description");
    syn::parse_file(&rust).expect("self-description Rust parses");
}

// ---------------------------------------------------------------------------
// Complex kind tests
// ---------------------------------------------------------------------------

#[test]
fn complex_kind_reads_superkinds_and_associated_types() {
    let source = "Library.{0 1 0} [] [] [ Streamable.{ [ Fillable ] [ Item ] \u{00AB}\u{00BB} [ next![ Option<Item> ] ] } ] []";
    let concept = Potential::from(source).actualize().expect("complex kind");
    let Concept::Library(library) = &concept else {
        panic!("expected Library");
    };
    match &library.kinds[0] {
        ethos_zero::KindDeclaration::Complex {
            name,
            superkinds,
            associated_types,
            capabilities,
            ..
        } => {
            assert_eq!(name, "Streamable");
            assert_eq!(superkinds, &["Fillable"]);
            assert_eq!(associated_types.len(), 1);
            assert_eq!(associated_types[0].name, "Item");
            assert_eq!(capabilities.len(), 1);
        }
        _ => panic!("expected Complex kind"),
    }
}

#[test]
fn intrinsic_names_emit_fully_qualified() {
    let source = "Library.{0 1 0} [] [ Pair.{ Integer Decimal Boolean } ] [] []";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(
        rust.contains("protos :: Integer"),
        "Integer not fully qualified in: {rust}"
    );
    assert!(
        rust.contains("protos :: Decimal"),
        "Decimal not fully qualified in: {rust}"
    );
    assert!(
        rust.contains("protos :: Boolean"),
        "Boolean not fully qualified in: {rust}"
    );
}

#[test]
fn bootstrap_module_is_fresh() {
    let source = fs::read_to_string("ethos-zero.ethos").expect("self-description");
    let concept = Potential::from(source.as_str())
        .actualize()
        .expect("read self-description");
    let emitted = concept.emit().expect("emit self-description");
    let committed = fs::read_to_string("src/generated.rs").expect("committed generated.rs");
    assert_eq!(
        emitted, committed,
        "src/generated.rs is stale: re-run the emitter on ethos-zero.ethos"
    );
}

// ---------------------------------------------------------------------------
// Protosizable round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn library_fixture_round_trips_through_protosize() {
    let source = fs::read_to_string("fixtures/example-library.ethos").expect("fixture");
    let concept = Potential::from(source.as_str()).actualize().expect("read");
    let printed = concept.protosize().print();
    let round_tripped = Potential::from(printed.as_str())
        .actualize()
        .expect("read round-tripped");
    assert_eq!(concept, round_tripped, "round trip changed the concept");
}

#[test]
fn signal_fixture_round_trips_through_protosize() {
    let source = fs::read_to_string("fixtures/orchestrate.ethos").expect("fixture");
    let concept = Potential::from(source.as_str()).actualize().expect("read");
    let printed = concept.protosize().print();
    let round_tripped = Potential::from(printed.as_str())
        .actualize()
        .expect("read round-tripped");
    assert_eq!(concept, round_tripped, "round trip changed the concept");
}

#[test]
fn self_description_round_trips_through_protosize() {
    let source = fs::read_to_string("ethos-zero.ethos").expect("self-description");
    let concept = Potential::from(source.as_str()).actualize().expect("read");
    let printed = concept.protosize().print();
    let round_tripped = Potential::from(printed.as_str())
        .actualize()
        .expect("read round-tripped");
    assert_eq!(concept, round_tripped, "round trip changed the concept");
}

#[test]
fn canonical_file_prints_to_itself() {
    // A file in the full form should print canonically and read back the same
    let source = "Library.{ { 0 1 0 } [] [ Pair.{ Text Integer } ] [] [] }";
    let concept = Potential::from(source).actualize().expect("read");
    let printed = concept.protosize().print();
    // The printed text should read back to the same concept
    let round_tripped = Potential::from(printed.as_str())
        .actualize()
        .expect("read round-tripped");
    assert_eq!(concept, round_tripped);
}

// ---------------------------------------------------------------------------
// End-to-end compile test
// ---------------------------------------------------------------------------

#[test]
fn fixture_signal_generated_rust_compiles_and_round_trips_values() {
    use std::process::Command;

    // Skip in sandboxed Nix builds: the isolated Cargo project needs network
    // access for rkyv from crates.io. The flake's other checks (build, clippy)
    // already compile the generated code at the crate level.
    if std::env::var("NIX_BUILD_TOP").is_ok() {
        eprintln!("skipping e2e compile test inside Nix sandbox");
        return;
    }

    let source = fs::read_to_string("fixtures/orchestrate.ethos").expect("fixture");
    let concept = Potential::from(source.as_str()).actualize().expect("read");
    let rust = concept.emit().expect("emit");

    // Create an isolated Cargo project in a temp directory
    let dir = std::env::temp_dir().join("ethos-zero-e2e-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("create dir");

    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "e2e-test"
version = "0.1.0"
edition = "2024"
rust-version = "1.89"

[dependencies]
protos = { git = "https://github.com/LiGoldragon/protos", rev = "56c683ec8d1e" }
datomic = { git = "https://github.com/LiGoldragon/datomic", rev = "768426ea5f34" }
rkyv = { version = "0.8", default-features = false, features = ["std", "bytecheck", "little_endian", "pointer_width_32", "unaligned"] }
"#,
    )
    .expect("write Cargo.toml");

    // Write the generated Rust as a module
    fs::write(dir.join("src").join("generated.rs"), &rust).expect("write generated.rs");

    // Write main.rs that uses the generated code and round-trips values
    fs::write(
        dir.join("src").join("main.rs"),
        r#"
mod generated;
use generated::*;
use datomic::{Corporal, Datomic, Textualizable};
use protos::Structural;

fn round_trip<T: Datomic + std::fmt::Debug + PartialEq>(value: T, expected_text: &str) {
    let text = value.textualize();
    assert_eq!(text.as_str(), expected_text, "textualize mismatch for {value:?}");
    let delineation = text.delineate().unwrap();
    use protos::Conceptual;
    let datom: datomic::Datom = delineation.conceive().unwrap();
    let recovered = <T as Corporal<datomic::Datom>>::incorporate(datom).unwrap();
    assert_eq!(value, recovered, "round trip failed for {expected_text}");
}

fn main() {
    // Lock round-trip (aliases are type aliases, not wrappers)
    round_trip(
        Lock(42, "MyLock".to_owned(), "6329f1".to_owned(),
             vec!["/abs/path".to_owned()], "testing".to_owned()),
        "{ 42 MyLock 6329f1 [ /abs/path ] testing }",
    );

    // Release.42 round-trip (LockId is Integer, bare in datom)
    round_trip(
        Request::Release(42),
        "Release.42",
    );

    // LockRequest round-trip
    round_trip(
        LockRequest("Test".to_owned(), "abc".to_owned(), vec![], "reason".to_owned()),
        "{ Test abc [] reason }",
    );

    // Observed.Locks.[] round-trip
    round_trip(
        Reply::Observed(Observation::Locks(vec![])),
        "Observed.Locks.[]",
    );

    // ReleaseRejected.UnknownLockId round-trip
    round_trip(
        Reply::ReleaseRejected(ReleaseRejection::UnknownLockId),
        "ReleaseRejected.UnknownLockId",
    );

    // Wire Refusal round-trip through datom text
    round_trip(
        Refusal::VersionMismatch(Version(1, 0, 0), Version(0, 9, 0)),
        "VersionMismatch.{ { 1 0 0 } { 0 9 0 } }",
    );
    round_trip(
        Refusal::Unreadable,
        "Unreadable",
    );

    println!("All round-trips passed.");
}
"#,
    )
    .expect("write main.rs");

    // Build the project
    let build = Command::new("cargo")
        .args(["build"])
        .current_dir(&dir)
        .output()
        .expect("cargo build");
    assert!(
        build.status.success(),
        "cargo build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Run the project
    let run = Command::new("cargo")
        .args(["run"])
        .current_dir(&dir)
        .output()
        .expect("cargo run");
    assert!(
        run.status.success(),
        "cargo run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    // Cleanup
    let _ = fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Import resolution tests
// ---------------------------------------------------------------------------

#[test]
fn imported_name_emits_qualified_in_struct_field() {
    let source = "\
Library.{0 1 0}
[datomic:Fault]
[ Wrapper.{ Fault } ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(
        rust.contains("datomic :: Fault"),
        "imported Fault should be qualified as datomic::Fault in: {rust}"
    );
}

#[test]
fn imported_names_emit_qualified_in_enum_variant() {
    let source = "\
Library.{0 1 0}
[datomic:[ Fault ]]
[ Error.[ Bad.Fault Good ] ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(
        rust.contains("datomic :: Fault"),
        "imported Fault should be qualified: {rust}"
    );
    syn::parse_file(&rust).expect("generated Rust parses");
}

#[test]
fn imported_generic_constructor_emits_qualified() {
    let source = "\
Library.{0 1 0}
[custom_crate:[ Container Item ]]
[ Holder.Container<Item> ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(
        rust.contains("custom_crate :: Container"),
        "imported constructor should be qualified: {rust}"
    );
    assert!(
        rust.contains("custom_crate :: Item"),
        "imported argument should be qualified: {rust}"
    );
}

#[test]
fn unimported_names_stay_bare() {
    let source = "\
Library.{0 1 0}
[]
[ Pair.{ Foo Bar } ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    // Should NOT contain module-qualified Foo or Bar
    assert!(
        !rust.contains(":: Foo") && !rust.contains(":: Bar"),
        "unimported names should not be qualified: {rust}"
    );
}

#[test]
fn intrinsic_names_not_overridden_by_imports() {
    // Even if someone imports Text from a custom source, the intrinsic mapping wins
    let source = "\
Library.{0 1 0}
[custom:Text]
[ Wrapper.{ Text } ]
[]
[]";
    let concept = Potential::from(source).actualize().expect("read");
    let rust = concept.emit().expect("emit");
    assert!(
        rust.contains("protos :: Text"),
        "intrinsic Text should still be protos::Text: {rust}"
    );
}

// ---------------------------------------------------------------------------
// Datom round-trip proptest
// ---------------------------------------------------------------------------

fn arb_type_expression() -> impl Strategy<Value = ethos_zero::TypeExpression> {
    let leaf = prop_oneof![
        Just(ethos_zero::TypeExpression::Named("Text".to_owned())),
        Just(ethos_zero::TypeExpression::Named("Integer".to_owned())),
        Just(ethos_zero::TypeExpression::Named("Boolean".to_owned())),
    ];
    leaf.prop_recursive(2, 8, 4, |inner| {
        prop_oneof![
            inner
                .clone()
                .prop_map(|t| ethos_zero::TypeExpression::Applied {
                    constructor: "Vector".to_owned(),
                    arguments: vec![t],
                }),
            inner
                .clone()
                .prop_map(|t| ethos_zero::TypeExpression::Applied {
                    constructor: "Option".to_owned(),
                    arguments: vec![t],
                }),
        ]
    })
}

fn arb_variant() -> impl Strategy<Value = ethos_zero::Variant> {
    prop_oneof![
        "[A-Z][a-z]{2,6}".prop_map(ethos_zero::Variant::Unit),
        ("[A-Z][a-z]{2,6}", arb_type_expression())
            .prop_map(|(n, t)| ethos_zero::Variant::Typed(n, t)),
    ]
}

fn arb_type_declaration() -> impl Strategy<Value = ethos_zero::TypeDeclaration> {
    prop_oneof![
        (
            "[A-Z][a-z]{2,6}",
            proptest::collection::vec(arb_type_expression(), 1..4)
        )
            .prop_map(|(n, f)| ethos_zero::TypeDeclaration::Struct { name: n, fields: f }),
        (
            "[A-Z][a-z]{2,6}",
            proptest::collection::vec(arb_variant(), 1..4)
        )
            .prop_map(|(n, v)| ethos_zero::TypeDeclaration::Enum {
                name: n,
                variants: v
            }),
        ("[A-Z][a-z]{2,6}", arb_type_expression())
            .prop_map(|(n, t)| ethos_zero::TypeDeclaration::Alias { name: n, target: t }),
    ]
}

fn arb_library() -> impl Strategy<Value = ethos_zero::Concept> {
    (
        (0i64..10, 0i64..10, 0i64..10),
        proptest::collection::vec(arb_type_declaration(), 0..5),
    )
        .prop_map(|((major, minor, patch), types)| {
            ethos_zero::Concept::Library(ethos_zero::Library {
                version: ethos_zero::Version(major, minor, patch),
                imports: vec![],
                types,
                kinds: vec![],
                associations: vec![],
            })
        })
}

proptest! {
    #[test]
    fn concept_protosize_round_trips(concept in arb_library()) {
        let printed = concept.protosize().print();
        let round_tripped = Potential::from(printed.as_str())
            .actualize()
            .map_err(|e| TestCaseError::Fail(format!("actualize failed: {e}").into()))?;
        prop_assert_eq!(concept, round_tripped, "round trip changed the concept");
    }
}
