use ethos_zero::{Actualizing, Concept, Emitting, Potential, Version};
use std::fs;

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
