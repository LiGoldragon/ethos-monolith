use ethos_zero::{FileReader, Manifest, RustEmitter};
use quote::ToTokens;
use std::{fs, process::Command};

struct EmptyManifest;

impl Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<ethos_zero::FileLocation> {
        None
    }
}

struct OneSourceManifest;

impl Manifest for OneSourceManifest {
    fn resolve(&self, source: &str) -> Option<ethos_zero::FileLocation> {
        (source == "outside").then(|| ethos_zero::FileLocation {
            directory: "vendor".into(),
            file: "outside.ethos".into(),
        })
    }
}

#[test]
fn schema_root_is_distinct_and_legacy_headerless_text_is_rejected() {
    let source = fs::read_to_string("ethos-zero.ethos").expect("E0 map");
    let file = FileReader::new(&EmptyManifest)
        .read(&source)
        .expect("schema root");
    let rust = RustEmitter::new().emit(&file).expect("emit Rust");
    syn::parse_file(&rust).expect("generated Rust parses");
    assert!(FileReader::new(&EmptyManifest).read("[] [] []").is_err());
}

#[test]
fn unresolvable_source_import_faults_without_a_local_fallback() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [outside:Thing] {[] [] [] [] []}";
    let error = FileReader::new(&EmptyManifest)
        .read(source)
        .expect_err("manifest resolution");
    assert_eq!(error.reason, ethos_zero::FileFaultReason::UnresolvedImport);
}

#[test]
fn all_canonical_import_shapes_are_portion_matched_and_manifest_resolved() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [outside:Thing outside:[One Two] outside:file.[Three] outside:dir/file.[Four] local/dir/file.[Local]] {[] [] [] [] []}";
    let file = FileReader::new(&OneSourceManifest)
        .read(source)
        .expect("canonical imports");
    let ethos_zero::File::Interface(interface) = file else {
        panic!("interface");
    };
    assert_eq!(interface.imports.len(), 5);
    assert_eq!(interface.imports[0].location.file, "outside.ethos");
    assert_eq!(interface.imports[4].location.file, "local/dir/file");
}

#[test]
fn datomic_manifest_is_the_concrete_external_import_index() {
    let manifest = ethos_zero::DatomicManifest::embody("«outside vendor/dir/file.ethos»")
        .expect("Datomic manifest");
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [outside:Thing] {[] [] [] [] []}";
    let file = FileReader::new(&manifest)
        .read(source)
        .expect("manifest resolution");
    let ethos_zero::File::Interface(interface) = file else {
        panic!("interface");
    };
    assert_eq!(interface.imports[0].location.directory, "vendor/dir");
    assert_eq!(interface.imports[0].location.file, "file.ethos");
}

#[test]
fn source_linked_protos_and_datomic_maps_read_and_emit_through_the_portion_pivot() {
    let reader = FileReader::new(&EmptyManifest);
    for variable in ["ETHOS_PROTOS_MAP", "ETHOS_DATOMIC_MAP"] {
        let path = std::env::var(variable).expect("Nix supplies the pinned authored map path");
        let source = fs::read_to_string(path).expect("authored map");
        let file = reader.read(&source).expect("map embodiment");
        assert!(matches!(file, ethos_zero::File::Schema(_)));
        let emitted = RustEmitter::new()
            .emit(&file)
            .expect("map syn/quote emission");
        syn::parse_file(&emitted).expect("emitted map Rust syntax");
    }
}

#[test]
fn map_owned_public_contracts_match_handwritten_sources() {
    let reader = FileReader::new(&EmptyManifest);
    let scopes = [
        (
            "ETHOS_PROTOS_MAP",
            "ETHOS_PROTOS_RUST",
            [
                "Text",
                "ContentHash",
                "Symbol",
                "Extent",
                "Separator",
                "BareExpectation",
                "Enclosure",
                "StructuralEnclosure",
                "OpaqueBoundary",
                "Boundary",
                "DialectBoundary",
                "Portion",
                "Headed",
                "Enclosed",
                "StructuralEnclosed",
                "OpaqueEnclosed",
                "Bare",
                "Delineation",
                "Fault",
                "FaultProblem",
                "Layout",
                "Prospective",
                "Delineatable",
                "Embodiable",
                "Embodied",
                "Textualizable",
                "ShapeDefined",
                "ContentHashable",
                "BareSafe",
                "PortionText",
                "ScalarAnatomy",
                "EnclosedArity",
                "EnclosedAnatomy",
                "Printing",
                "DelineatedText",
            ]
            .as_slice(),
        ),
        (
            "ETHOS_DATOMIC_MAP",
            "ETHOS_DATOMIC_RUST",
            [
                "Fault",
                "FaultProblem",
                "FiniteDecimal",
                "DatomicString",
                "NonFiniteDecimal",
                "UnrepresentableString",
                "Datomic",
                "TextEdge",
                "PortionViewing",
                "DecimalViewing",
                "PortionBuilding",
            ]
            .as_slice(),
        ),
    ];
    for (map_variable, rust_variable, names) in scopes {
        let map = fs::read_to_string(std::env::var(map_variable).expect("pinned map"))
            .expect("map source");
        let generated = RustEmitter::schema_library()
            .emit(&reader.read(&map).expect("map embodiment"))
            .expect("map emission");
        let handwritten = fs::read_to_string(std::env::var(rust_variable).expect("pinned Rust"))
            .expect("handwritten source");
        let generated = rustfmt_for_comparison(&generated);
        let handwritten = rustfmt_for_comparison(&handwritten);
        let generated = syn::parse_file(&generated).expect("generated syntax");
        let handwritten = syn::parse_file(&handwritten).expect("handwritten syntax");
        for name in names {
            assert_eq!(
                contract_projection(&generated, name),
                contract_projection(&handwritten, name),
                "map-owned public item {name}"
            );
        }
    }
}

#[test]
fn complete_map_owned_declarations_splice_into_the_real_engines() {
    let reader = FileReader::new(&EmptyManifest);
    for (map_variable, rust_variable) in [
        ("ETHOS_PROTOS_MAP", "ETHOS_PROTOS_RUST"),
        ("ETHOS_DATOMIC_MAP", "ETHOS_DATOMIC_RUST"),
    ] {
        let map = fs::read_to_string(std::env::var(map_variable).expect("pinned map"))
            .expect("map source");
        let file = reader.read(&map).expect("map embodiment");
        let generated = RustEmitter::schema_library()
            .generate(&file)
            .expect("map emission");
        let names = map_owned_names(&file);
        let rust_path = std::path::PathBuf::from(std::env::var(rust_variable).expect("engine"));
        let mut engine = syn::parse_file(&fs::read_to_string(&rust_path).expect("engine source"))
            .expect("engine syntax");
        engine
            .items
            .retain(|item| !is_map_owned_declaration(item, &names));
        engine.items.extend(generated.syntax.items);

        let directory = std::env::temp_dir().join(format!(
            "ethos-zero-map-splice-{}-{}",
            std::process::id(),
            rust_path.parent().expect("source directory").display()
        ));
        let source_directory = directory.join("src");
        fs::create_dir_all(&source_directory).expect("splice source directory");
        fs::write(
            directory.join("Cargo.toml"),
            engine_manifest(rust_variable == "ETHOS_DATOMIC_RUST"),
        )
        .expect("splice manifest");
        fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("splice lockfile");
        fs::write(
            source_directory.join("lib.rs"),
            engine.into_token_stream().to_string(),
        )
        .expect("spliced engine source");
        assert!(
            Command::new("cargo")
                .args(["check", "--offline"])
                .current_dir(&directory)
                .status()
                .expect("spliced engine cargo invocation")
                .success(),
            "all map-owned declarations must compile when spliced into the real engine"
        );
        fs::remove_dir_all(directory).expect("splice cleanup");
    }
}

#[test]
fn generated_schema_defaults_execute_through_the_real_engines() {
    let reader = FileReader::new(&EmptyManifest);
    for (name, map_variable, rust_variable, program) in [
        (
            "protos",
            "ETHOS_PROTOS_MAP",
            "ETHOS_PROTOS_RUST",
            r#"
use ethos_zero_map_witness::{Embodied, Fault, Portion, Textualizable};

struct Witness;

impl Embodied for Witness {
    fn from_portion(_: &Portion) -> Result<Self, Fault> {
        Ok(Self)
    }
}

impl Textualizable for Witness {
    fn to_portion(&self) -> Portion {
        Portion::from_expected_string("generated-default")
            .expect("a representable Protos witness")
    }
}

fn main() {
    assert_eq!(Textualizable::textualize(&Witness).as_ref(), "generated-default");
}
"#,
        ),
        (
            "datomic",
            "ETHOS_DATOMIC_MAP",
            "ETHOS_DATOMIC_RUST",
            r#"
use datomic::{Datomic, DatomicString};

fn main() {
    let value = DatomicString::try_from("generated default".to_owned())
        .expect("a representable Datomic witness");
    assert_eq!(value.textualize().as_ref(), "“generated default”");
}
"#,
        ),
    ] {
        let map = fs::read_to_string(std::env::var(map_variable).expect("pinned map"))
            .expect("map source");
        let file = reader.read(&map).expect("map embodiment");
        let generated = RustEmitter::schema_library()
            .generate(&file)
            .expect("map emission");
        let names = map_owned_names(&file);
        let rust_path = std::path::PathBuf::from(std::env::var(rust_variable).expect("engine"));
        let mut engine = syn::parse_file(&fs::read_to_string(&rust_path).expect("engine source"))
            .expect("engine syntax");
        engine
            .items
            .retain(|item| !is_map_owned_declaration(item, &names));
        engine.items.extend(generated.syntax.items);

        let directory = std::env::temp_dir().join(format!(
            "ethos-zero-generated-default-{name}-{}",
            std::process::id()
        ));
        let source_directory = directory.join("src");
        fs::create_dir_all(&source_directory).expect("witness source directory");
        fs::write(
            directory.join("Cargo.toml"),
            engine_manifest(name == "datomic"),
        )
        .expect("witness manifest");
        fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("witness lockfile");
        fs::write(
            source_directory.join("lib.rs"),
            engine.into_token_stream().to_string(),
        )
        .expect("spliced engine source");
        fs::write(source_directory.join("main.rs"), program).expect("witness program");
        let target = directory.join("target");
        let output = Command::new("timeout")
            .args(["10", "cargo", "run", "--offline", "--quiet"])
            .env("CARGO_TARGET_DIR", &target)
            .current_dir(&directory)
            .output()
            .expect("bounded generated-default invocation");
        assert!(
            output.status.success(),
            "generated {name} default must complete: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(directory).expect("witness cleanup");
    }
}

#[test]
fn removing_the_real_text_content_hashable_impl_fails_the_emitted_association() {
    let reader = FileReader::new(&EmptyManifest);
    let map = fs::read_to_string(std::env::var("ETHOS_PROTOS_MAP").expect("pinned map"))
        .expect("map source");
    let file = reader.read(&map).expect("map embodiment");
    let generated = RustEmitter::schema_library()
        .generate(&file)
        .expect("map emission");
    let names = map_owned_names(&file);
    let rust_path = std::path::PathBuf::from(std::env::var("ETHOS_PROTOS_RUST").expect("engine"));
    let mut engine = syn::parse_file(&fs::read_to_string(&rust_path).expect("engine source"))
        .expect("engine syntax");
    engine.items.retain(|item| {
        !is_map_owned_declaration(item, &names) && !is_text_content_hashable_impl(item)
    });
    engine.items.extend(generated.syntax.items);

    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-negative-content-hashable-{}",
        std::process::id()
    ));
    let source_directory = directory.join("src");
    fs::create_dir_all(&source_directory).expect("negative source directory");
    fs::write(directory.join("Cargo.toml"), engine_manifest(false)).expect("negative manifest");
    fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("negative lockfile");
    fs::write(
        source_directory.join("lib.rs"),
        engine.into_token_stream().to_string(),
    )
    .expect("negative spliced source");
    let output = Command::new("cargo")
        .args(["check", "--offline"])
        .current_dir(&directory)
        .output()
        .expect("negative cargo invocation");
    assert!(
        !output.status.success(),
        "removing impl<T> ContentHashable for Text<T> must fail an emitted map association"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ContentHashable"),
        "the emitted ContentHashable association must be the failing contract"
    );
    fs::remove_dir_all(directory).expect("negative cleanup");
}

fn map_owned_names(file: &ethos_zero::File) -> Vec<String> {
    let ethos_zero::File::Schema(schema) = file else {
        panic!("map must be a schema");
    };
    schema
        .types
        .iter()
        .map(declaration_name)
        .chain(schema.kinds.iter().map(|kind| kind.name.clone()))
        .collect()
}

fn declaration_name(declaration: &ethos_zero::TypeDeclaration) -> String {
    match declaration {
        ethos_zero::TypeDeclaration::Alias { name, .. }
        | ethos_zero::TypeDeclaration::Struct { name, .. }
        | ethos_zero::TypeDeclaration::TupleStruct { name, .. }
        | ethos_zero::TypeDeclaration::Enum { name, .. } => name.clone(),
    }
}

fn is_map_owned_declaration(item: &syn::Item, names: &[String]) -> bool {
    let name = match item {
        syn::Item::Struct(item) => &item.ident,
        syn::Item::Enum(item) => &item.ident,
        syn::Item::Trait(item) => &item.ident,
        syn::Item::Type(item) => &item.ident,
        _ => return false,
    };
    names.iter().any(|candidate| name == candidate)
}

fn is_text_content_hashable_impl(item: &syn::Item) -> bool {
    let syn::Item::Impl(item) = item else {
        return false;
    };
    let Some((trait_path, _)) = &item.trait_ else {
        return false;
    };
    let syn::Type::Path(self_type) = item.self_ty.as_ref() else {
        return false;
    };
    trait_path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "ContentHashable")
        && self_type
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Text")
}

fn engine_manifest(needs_datomic: bool) -> String {
    let protos = std::env::var("ETHOS_PROTOS_CRATE").expect("pinned Protos crate path");
    let datomic = std::env::var("ETHOS_DATOMIC_CRATE").expect("pinned Datomic crate path");
    let datomic = needs_datomic.then(|| format!("datomic = {{ path = {datomic:?} }}\n"));
    format!(
        "[package]\nname = \"ethos-zero-map-witness\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nprotos = {{ path = {protos:?} }}\n{}[patch.\"https://github.com/LiGoldragon/protos\"]\nprotos = {{ path = {protos:?} }}\n",
        datomic.unwrap_or_default()
    )
}

fn wire_contract_manifest(name: &str) -> String {
    engine_manifest(true)
        .replace("ethos-zero-map-witness", name)
        .replace(
            "[dependencies]\n",
            "[dependencies]\nrkyv = { version = \"0.8\", default-features = false, features = [\"std\", \"bytecheck\", \"little_endian\", \"pointer_width_32\", \"unaligned\"] }\n",
        )
}

fn fixture_lockfile() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
        .expect("Ethos-zero locked dependency graph")
}

fn rustfmt_for_comparison(source: &str) -> String {
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-source-comparison-{}-{}",
        std::process::id(),
        source.len()
    ));
    fs::create_dir_all(&directory).expect("comparison directory");
    let path = directory.join("source.rs");
    fs::write(&path, source).expect("comparison source");
    assert!(
        Command::new("rustfmt")
            .args(["--edition", "2024"])
            .arg(&path)
            .status()
            .expect("rustfmt invocation")
            .success(),
        "comparison formatting"
    );
    let formatted = fs::read_to_string(&path).expect("formatted source");
    fs::remove_dir_all(directory).expect("comparison cleanup");
    formatted
}

fn contract_projection(file: &syn::File, name: &str) -> String {
    let mut item = file
        .items
        .iter()
        .find(|item| match item {
            syn::Item::Struct(item) => item.ident == name,
            syn::Item::Enum(item) => item.ident == name,
            syn::Item::Trait(item) => item.ident == name,
            syn::Item::Type(item) => item.ident == name,
            _ => false,
        })
        .cloned()
        .unwrap_or_else(|| panic!("public item {name}"));
    match &mut item {
        syn::Item::Struct(item) => {
            item.attrs
                .retain(|attribute| attribute.path().is_ident("non_exhaustive"));
        }
        syn::Item::Enum(item) => item
            .attrs
            .retain(|attribute| attribute.path().is_ident("non_exhaustive")),
        syn::Item::Trait(item) => {
            item.attrs.clear();
            for trait_item in &mut item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    method.sig.inputs.pop_punct();
                    if method.default.is_some() {
                        method.default = Some(syn::parse_quote!({}));
                    }
                    for (index, argument) in method.sig.inputs.iter_mut().enumerate() {
                        if let syn::FnArg::Typed(argument) = argument {
                            let name = syn::Ident::new(
                                &format!("input_{index}"),
                                proc_macro2::Span::call_site(),
                            );
                            *argument.pat = syn::parse_quote!(#name);
                        }
                    }
                }
            }
        }
        syn::Item::Type(item) => item.attrs.clear(),
        _ => unreachable!("selected structural item"),
    }
    item.to_token_stream()
        .to_string()
        .replace("protos :: ", "")
        .replace("datomic :: ", "")
}

#[test]
fn vector_and_result_type_applications_are_projected_without_rust_source_assembly() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[] [] [] [] [Things.Vector<Thing> Outcome.Result<Things Reason>]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("application read");
    let rust = RustEmitter::new().emit(&file).expect("application emit");
    assert!(rust.contains("Vec < Thing >"));
    assert!(rust.contains("Result < Things , Reason >"));
}

#[test]
fn unsupported_application_reaches_typed_projection_before_it_faults() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[] [] [] [] [Things.Set<Thing>]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("application still reads");
    let error = RustEmitter::new()
        .emit(&file)
        .expect_err("unsupported projection");
    assert_eq!(
        error.reason,
        ethos_zero::FileFaultReason::UnsupportedApplication
    );
}

#[test]
fn inline_payloads_become_deterministically_named_declarations() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[] [] [] [] [Name.String Message.[Record.{Name.String} Choice.[First Second]]]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("inline payload read");
    let rust = RustEmitter::new().emit(&file).expect("inline payload emit");
    assert!(rust.contains("struct MessageRecord"));
    assert!(rust.contains("enum MessageChoice"));
    assert!(rust.contains("Record (MessageRecord)"));
    assert!(rust.contains("Choice (MessageChoice)"));
}

#[test]
fn datomic_schema_reaches_executable_anatomy_without_a_stub() {
    let source = "Schema.{0 1 0} [] Types.[Fault.{Extent.String} Value.String Record.{Value} Mode.[Unit Data.Record] List.Vector<Value> Optional.Option<Value> Mapping.«Key.String Value.String»] Kinds.[Datomic.{embody.[Result<Self Fault>] portion.[Portion] textualize.[Text<Self>]}] Associations.[]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("Datomic map read");
    let rust = RustEmitter::new().emit(&file).expect("D3 anatomy emission");
    syn::parse_file(&rust).expect("D3 syntax");
    assert!(rust.contains("impl datomic :: Datomic for Record"));
}

#[test]
fn schema_kinds_and_associations_become_traits_and_checked_impls() {
    let source = "Schema.{0 1 0} [] Types.[Name.String User.{Name}] Kinds.[Processable.[]] Associations.[User.[Processable]]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("schema read");
    let rust = RustEmitter::new().emit(&file).expect("schema emit");
    assert!(rust.contains("trait Processable"));
    assert!(rust.contains("fn carries < T > () where T : Processable"));
    syn::parse_file(&rust).expect("syntax");
}

#[test]
fn false_kind_association_fails_generated_rust_compilation() {
    let source = "Schema.{0 1 0} [] Types.[User.{}] Kinds.[] Associations.[User.[Missing]]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("schema read");
    let rust = RustEmitter::new().emit(&file).expect("schema emit");
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-false-association-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("isolated output directory");
    let source_path = directory.join("generated.rs");
    fs::write(&source_path, rust).expect("generated Rust source");
    let output = Command::new("rustc")
        .args(["--edition", "2024", "--crate-type", "lib"])
        .arg(&source_path)
        .arg("--out-dir")
        .arg(&directory)
        .output()
        .expect("rustc invocation");
    assert!(!output.status.success(), "missing association must fail");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Missing"));
    fs::remove_dir_all(directory).expect("isolated output cleanup");
}

#[test]
fn pinned_orchestrate_interfaces_are_read_from_their_real_sources() {
    let reader = FileReader::new(&EmptyManifest);
    for (variable, channel, contract, wire) in [
        ("ETHOS_SIGNAL_ORCHESTRATE_SOURCE", "Orchestrate", 1, 5),
        (
            "ETHOS_META_SIGNAL_ORCHESTRATE_SOURCE",
            "MetaOrchestrate",
            2,
            4,
        ),
    ] {
        let source = fs::read_to_string(std::env::var(variable).expect("pinned source path"))
            .expect("pinned authored interface");
        let file = reader.read(&source).expect("complete headed interface");
        let ethos_zero::File::Interface(interface) = file else {
            panic!("pinned source must be an interface");
        };
        assert_eq!(interface.channel.name, channel);
        assert_eq!(interface.channel.contract, contract);
        assert_eq!(interface.channel.wire, wire);
    }
}

#[test]
fn generated_real_orchestrate_interfaces_compile_and_exercise_their_source_linked_types() {
    let reader = FileReader::new(&EmptyManifest);
    let interfaces = [
        (
            "ordinary",
            "ETHOS_SIGNAL_ORCHESTRATE_SOURCE",
            r#"
use datomic::{Datomic, DatomicString, Text, TextEdge};
use generated_ordinary::*;

fn string(value: &str) -> DatomicString {
    DatomicString::try_from(value.to_owned()).expect("representable fixture string")
}

#[test]
fn approved_operations_and_string_edges_round_trip() {
    let lock_name: LockName = Text::<LockName>::from("<Kind>")
        .embody()
        .expect("LockName angle string");
    assert_eq!(lock_name.textualize().as_ref(), "<Kind>");
    let lock_reason: LockReason = Text::<LockReason>::from("(outer (nested) tail)")
        .embody()
        .expect("LockReason parenthesized string");
    assert_eq!(lock_reason.textualize().as_ref(), "“outer (nested) tail”");

    let lock = Lock {
        lock_id: 7,
        lock_name,
        flow_id: string("01a04a30"),
        lock_paths: vec![string("/tmp/ethos-zero-e2")],
        lock_reason,
    };
    let text = lock.textualize();
    assert_eq!(Text::<Lock>::from(text.as_ref()).embody().expect("Lock embodiment").textualize().as_ref(), text.as_ref());

    let request = Request::Lock(LockRequest {
        lock_name: string("e2"),
        flow_id: string("01a04a30"),
        lock_paths: vec![],
        lock_reason: string("acceptance"),
    });
    assert!(matches!(request, Request::Lock(_)));
    assert!(matches!(Reply::Locked(lock), Reply::Locked(_)));
    let duplicate = Lock {
        lock_id: 8,
        lock_name: string("duplicate"),
        flow_id: string("01a04a30"),
        lock_paths: vec![],
        lock_reason: string("duplicate name"),
    };
    assert!(matches!(Reply::LockRejected(LockRejection::DuplicateName(duplicate)), Reply::LockRejected(_)));
}
"#,
        ),
        (
            "meta",
            "ETHOS_META_SIGNAL_ORCHESTRATE_SOURCE",
            r#"
use datomic::{Datomic, DatomicString, Text, TextEdge};
use generated_meta::*;

fn string(value: &str) -> DatomicString {
    DatomicString::try_from(value.to_owned()).expect("representable fixture string")
}

#[test]
fn approved_configuration_operation_round_trips() {
    let configure = Configure {
        ordinary_socket_path: string("/tmp/orchestrate.sock"),
        meta_socket_path: string("/tmp/meta-orchestrate.sock"),
    };
    let text = configure.textualize();
    assert_eq!(Text::<Configure>::from(text.as_ref()).embody().expect("Configure embodiment").textualize().as_ref(), text.as_ref());
    assert!(matches!(Request::Configure(configure), Request::Configure(_)));
}
"#,
        ),
    ];
    for (name, variable, fixture) in interfaces {
        let source = fs::read_to_string(std::env::var(variable).expect("pinned source path"))
            .expect("pinned authored interface");
        let file = reader.read(&source).expect("pinned interface embodiment");
        let generated = RustEmitter::new()
            .emit(&file)
            .expect("pinned interface emission");
        compile_generated_interface(name, &generated, fixture);
    }
}

fn compile_generated_interface(name: &str, generated: &str, fixture: &str) {
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-real-interface-{name}-{}",
        std::process::id()
    ));
    let source_directory = directory.join("src");
    let test_directory = directory.join("tests");
    fs::create_dir_all(&source_directory).expect("generated source directory");
    fs::create_dir_all(&test_directory).expect("generated test directory");
    fs::write(
        directory.join("Cargo.toml"),
        engine_manifest(true).replace("ethos-zero-map-witness", &format!("generated-{name}")),
    )
    .expect("fixture manifest");
    fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("fixture lockfile");
    fs::write(source_directory.join("lib.rs"), generated).expect("generated Rust");
    fs::write(test_directory.join("round_trip.rs"), fixture).expect("fixture tests");
    let target = directory.join("target");
    assert!(
        Command::new("cargo")
            .args(["test", "--offline"])
            .env("CARGO_TARGET_DIR", &target)
            .current_dir(&directory)
            .status()
            .expect("fixture cargo invocation")
            .success(),
        "generated real interface must compile and exercise its source-linked types"
    );
    fs::remove_dir_all(directory).expect("fixture cleanup");
}

#[test]
fn interface_sections_lower_references_without_redeclaring_payload_types() {
    let reader = FileReader::new(&EmptyManifest);
    let ordinary = "Interface.{0 2 0} Channel.{Orchestrate 1 5} [] {[Lock.LockRequest] [] [] [] [LockRequest.{Name.String}]}";
    let rust = RustEmitter::new()
        .emit(&reader.read(ordinary).expect("ordinary interface"))
        .expect("ordinary emission");
    assert!(rust.contains("enum Request { Lock (LockRequest)"));
    assert!(!rust.contains("type Lock = LockRequest"));

    let meta = RustEmitter::new()
        .generate(
            &reader
                .read("Interface.{0 1 0} Channel.{Meta 1 0} [] {[Configure.Configure] [] [] [] [Configure.{Path.String}]}")
                .expect("meta interface"),
        )
        .expect("meta emission")
        .syntax;
    assert_eq!(
        meta.items
            .iter()
            .filter(|item| matches!(item, syn::Item::Struct(item) if item.ident == "Configure"))
            .count(),
        1,
        "Configure.Configure is a section reference, not a second declaration"
    );
}

#[test]
fn malformed_headerless_fixture_reports_its_exact_portion_extent() {
    let error = FileReader::new(&EmptyManifest)
        .read("Legacy")
        .expect_err("headerless input must not be accepted");
    assert_eq!(error.reason, ethos_zero::FileFaultReason::Root);
    assert_eq!(error.extent, protos::Extent { start: 0, end: 6 });
}

#[test]
fn schema_map_type_expressions_are_read_without_a_second_parser() {
    let source = "Schema.{0 1 0} [] Types.[Text.{Value.T<Vector>}] Kinds.[] Associations.[]";
    FileReader::new(&EmptyManifest)
        .read(source)
        .expect("generic map declaration");
}

#[test]
fn mixed_kind_capabilities_preserve_names_receivers_and_result_applications() {
    let source = "Schema.{0 1 0} [] Types.[Input.String Output.String Fault.String] Kinds.[Processable.<[Clone Send] Sized> Mutable.[rewrite![Output]] Simple.[observe.[Output]] Complex.[process.{[Input] [Result<Output Fault>]}]] Associations.[]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("mixed kind declaration");
    let rust = RustEmitter::new()
        .emit(&file)
        .expect("emit mixed kind declaration");
    assert!(rust.contains("trait Processable : Clone + Send + Sized"));
    assert!(rust.contains("fn process (& self , input_0 : Input) -> Result < Output , Fault >"));
    assert!(rust.contains("fn rewrite (& mut self) -> Output"));
    assert!(rust.contains("fn observe (& self) -> Output"));
    syn::parse_file(&rust).expect("mixed capability Rust syntax");
}

#[test]
fn datomic_schema_emits_concrete_anatomy_instead_of_an_unsupported_stub() {
    let source = "Schema.{0 1 0} [] Types.[Name.String Record.{Name} Choice.[Unit Data.Record] Values.Vector<Name> Maybe.Option<Name> Index.«Name Record»] Kinds.[Datomic.{embody.[Result<Self Fault>] portion.[Portion] textualize.[Text<Self>]}] Associations.[]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("Datomic declaration");
    let rust = RustEmitter::new().emit(&file).expect("D3 anatomy emission");
    assert!(rust.contains("impl datomic :: Datomic for Record"));
    assert!(rust.contains("impl datomic :: Datomic for Choice"));
    assert!(!rust.contains("unimplemented !"));
    syn::parse_file(&rust).expect("D3 anatomy Rust syntax");
}

#[test]
fn generated_orchestrate_anatomies_compile_and_round_trip_through_datomic() {
    let source = "Interface.{0 2 0} Channel.{Orchestrate 1 5} [] {[] [] [] [] [LockName.String FlowId.String LockPath.String LockPaths.Vector<LockPath> LockReason.String LockId.Integer Metadata.«LockName LockReason» MaybeLockId.Option<LockId> Lock.{LockId LockName FlowId LockPaths LockReason} LockRejection.[DuplicateName.Lock PathOverlap.{LockPath Lock}] ObserveSelection.[Locks] Audit.{LockPaths MaybeLockId Metadata}]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("Orchestrate declaration");
    let generated = RustEmitter::new().emit(&file).expect("D3 generation");
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-generated-orchestrate-{}",
        std::process::id()
    ));
    let source_directory = directory.join("src");
    let test_directory = directory.join("tests");
    fs::create_dir_all(&source_directory).expect("generated source directory");
    fs::create_dir_all(&test_directory).expect("generated test directory");
    fs::write(
        directory.join("Cargo.toml"),
        engine_manifest(true).replace("ethos-zero-map-witness", "generated-orchestrate"),
    )
    .expect("fixture manifest");
    fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("fixture lockfile");
    fs::write(source_directory.join("lib.rs"), generated).expect("generated Rust");
    fs::write(
        test_directory.join("round_trip.rs"),
        r#"
use datomic::{Datomic, DatomicString, Text, TextEdge};
use generated_orchestrate::*;
use std::collections::BTreeMap;

fn string(value: &str) -> DatomicString {
    DatomicString::try_from(value.to_owned()).expect("representable fixture string")
}

#[test]
fn approved_ordinary_fixture_round_trips() {
    let value = Lock {
        lock_id: 7,
        lock_name: string("ethos-zero-e2"),
        flow_id: string("db97561c"),
        lock_paths: vec![string("/tmp/lock")],
        lock_reason: string("generated"),
    };
    let text = value.textualize();
    let embodied = Text::<Lock>::from(text.as_ref()).embody().expect("embody");
    assert_eq!(embodied.textualize().as_ref(), text.as_ref());
}

#[test]
fn containers_and_headed_payloads_round_trip() {
    let audit = Audit {
        lock_paths: vec![string("one"), string("two")],
        maybe_lock_id: Some(3),
        metadata: BTreeMap::from([(string("kind"), string("core"))]),
    };
    let text = audit.textualize();
    let embodied = Text::<Audit>::from(text.as_ref()).embody().expect("embody");
    assert_eq!(embodied.textualize().as_ref(), text.as_ref());
    let rejection = LockRejection::PathOverlap(LockRejectionPathOverlap {
        lock_path: string("one"),
        lock: Lock {
            lock_id: 3,
            lock_name: string("one"),
            flow_id: string("flow"),
            lock_paths: vec![],
            lock_reason: string("reason"),
        },
    });
    let text = rejection.textualize();
    let embodied = Text::<LockRejection>::from(text.as_ref()).embody().expect("embody");
    assert_eq!(embodied.textualize().as_ref(), text.as_ref());
}
"#,
    )
    .expect("fixture tests");
    let target = directory.join("target");
    assert!(
        Command::new("cargo")
            .arg("test")
            .arg("--offline")
            .env("CARGO_TARGET_DIR", &target)
            .current_dir(&directory)
            .status()
            .expect("fixture cargo invocation")
            .success(),
        "generated Orchestrate anatomy must compile and round trip"
    );
    fs::remove_dir_all(directory).expect("fixture cleanup");
}

#[test]
fn interface_file_is_read_from_the_portion_pivot_and_emitted_as_rust_syntax() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[Create.User] [Created.User] [Refused.Reason] [] [Name.String Reason.String User.{Name}]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("read file");
    let rust = RustEmitter::new().emit(&file).expect("emit Rust");
    syn::parse_file(rust.as_ref()).expect("generated Rust parses");
}

#[test]
fn wire_contract_emission_generates_only_the_signal_module() {
    let source = "Interface.{0 2 0} Channel.{Example 7 3} [] {[Submit.Submission] [Submitted.Result] [] [] [Name.String Submission.{Name} Result.{Name}]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("wire map");
    let emitted = RustEmitter::wire_contract()
        .emit(&file)
        .expect("wire emission");
    let syntax = syn::parse_file(&emitted).expect("wire syntax");
    assert!(emitted.contains("derive (Archive"));
    assert!(emitted.contains("struct Name") && emitted.contains("String"));
    assert!(emitted.contains("enum FrameBody"));
    assert!(!emitted.contains("unimplemented"));
    assert!(!emitted.contains("Codec"));
    let name = syntax
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Struct(item) if item.ident == "Name" => Some(item),
            _ => None,
        })
        .expect("wire string alias declaration");
    let syn::Fields::Unnamed(fields) = &name.fields else {
        panic!("wire string alias stays a tuple newtype");
    };
    assert!(matches!(fields.unnamed[0].vis, syn::Visibility::Inherited));
    assert!(
        syntax
            .items
            .iter()
            .any(|item| matches!(item, syn::Item::Struct(item) if item.ident == "Frame"))
    );
    let directory =
        std::env::temp_dir().join(format!("ethos-zero-wire-contract-{}", std::process::id()));
    let source_directory = directory.join("src");
    let test_directory = directory.join("tests");
    fs::create_dir_all(&source_directory).expect("wire fixture source directory");
    fs::create_dir_all(&test_directory).expect("wire fixture test directory");
    fs::write(
        directory.join("Cargo.toml"),
        wire_contract_manifest("ethos-zero-wire-contract"),
    )
    .expect("wire fixture manifest");
    fs::write(source_directory.join("generated.rs"), emitted).expect("generated Signal module");
    assert!(
        !source_directory.join("generated.rs").exists()
            || source_directory.join("generated.rs").is_file(),
        "the generator owns one generated Signal module"
    );
    fs::write(
        source_directory.join("lib.rs"),
        "pub mod codec;\npub mod generated;\n",
    )
    .expect("signal crate boundary");
    fs::write(
        source_directory.join("codec.rs"),
        r#"
use crate::generated::{
    Frame, FrameBody, CHANNEL_CONTRACT_ID, CHANNEL_WIRE_REVISION, INTERFACE_VERSION,
};

pub struct HandOwnedCodec;

impl HandOwnedCodec {
    pub fn envelope(body: FrameBody) -> Frame {
        Frame {
            channel_contract_id: CHANNEL_CONTRACT_ID,
            channel_wire_revision: CHANNEL_WIRE_REVISION,
            protocol_version: INTERFACE_VERSION,
            body,
        }
    }
}
"#,
    )
    .expect("hand-owned codec module");
    fs::write(
        test_directory.join("signal_boundary.rs"),
        r#"
use ethos_zero_wire_contract::{
    codec::HandOwnedCodec,
    generated::{
        ChannelContractId, ChannelWireRevision, FrameBody, Name, ProtocolVersion, Request,
        Submission,
    },
};

#[test]
fn generated_signal_and_hand_owned_codec_meet_at_the_frame() {
    let frame = HandOwnedCodec::envelope(FrameBody::Request(Request::Submit(Submission {
        name: Name::try_from("ready").expect("representable fixture string"),
    })));
    assert_eq!(frame.channel_contract_id, ChannelContractId(7));
    assert_eq!(frame.channel_wire_revision, ChannelWireRevision(3));
    assert_eq!(frame.protocol_version, ProtocolVersion::new(0, 2, 0));
}
"#,
    )
    .expect("signal boundary witness");
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        directory.join("Cargo.lock"),
    )
    .expect("wire fixture lockfile");
    assert!(
        Command::new("cargo")
            .args(["test", "--offline"])
            .env("CARGO_TARGET_DIR", directory.join("target"))
            .current_dir(&directory)
            .status()
            .expect("wire fixture compile")
            .success(),
        "generated Signal module and hand-owned codec must compile together"
    );
    fs::remove_dir_all(directory).expect("wire fixture cleanup");
}

#[test]
fn wire_contract_rejects_tuple_structs_with_a_typed_fault() {
    let source = "Interface.{0 2 0} Channel.{Example 7 3} [] {[] [] [] [] [Token.Tuple.[Visibility.Public String]]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("wire tuple map");
    let error = RustEmitter::wire_contract()
        .emit(&file)
        .expect_err("tuple-struct wire declarations are unsupported");
    assert_eq!(error.reason, ethos_zero::FileFaultReason::Declaration);
}

#[test]
fn nonrepresentable_wire_string_is_rejected_before_projection() {
    let source = "Interface.{0 2 0} Channel.{Example 7 3} [] {[Submit.Submission] [Submitted.Result] [] [] [Name.String Submission.{Name} Result.{Name}]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("wire string map");
    let emitted = RustEmitter::wire_contract()
        .emit(&file)
        .expect("wire string emission");
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-wire-string-invariant-{}",
        std::process::id()
    ));
    let source_directory = directory.join("src");
    let test_directory = directory.join("tests");
    fs::create_dir_all(&source_directory).expect("wire string source directory");
    fs::create_dir_all(&test_directory).expect("wire string test directory");
    fs::write(
        directory.join("Cargo.toml"),
        wire_contract_manifest("ethos-zero-wire-string-invariant"),
    )
    .expect("wire string fixture manifest");
    fs::write(source_directory.join("generated.rs"), emitted).expect("generated wire module");
    fs::write(source_directory.join("lib.rs"), "pub mod generated;\n")
        .expect("wire string crate boundary");
    fs::write(
        test_directory.join("string_invariant.rs"),
        r#"
use ethos_zero_wire_string_invariant::generated::Name;
use datomic::Datomic;
use protos::PortionText;

#[test]
fn invalid_string_construction_is_rejected_without_projection() {
    assert!(Name::try_from("unbalanced “").is_err());
    let valid = Name::try_from("ready").expect("representable fixture string");
    assert_eq!(Datomic::portion(&valid).canonical_text().as_ref(), "ready");
}
"#,
    )
    .expect("wire string invariant witness");
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        directory.join("Cargo.lock"),
    )
    .expect("wire string fixture lockfile");
    assert!(
        Command::new("cargo")
            .args(["test", "--offline"])
            .env("CARGO_TARGET_DIR", directory.join("target"))
            .current_dir(&directory)
            .status()
            .expect("wire string fixture compile")
            .success(),
        "non-representable wire string must be rejected without a panic"
    );
    fs::remove_dir_all(directory).expect("wire string fixture cleanup");
}

#[test]
fn pinned_orchestrate_interfaces_emit_compilable_wire_contract_modules() {
    let reader = FileReader::new(&EmptyManifest);
    for (name, variable, contract, wire) in [
        ("ordinary", "ETHOS_SIGNAL_ORCHESTRATE_SOURCE", 1, 5),
        ("meta", "ETHOS_META_SIGNAL_ORCHESTRATE_SOURCE", 2, 4),
    ] {
        let source = fs::read_to_string(std::env::var(variable).expect("pinned source path"))
            .expect("pinned authored interface");
        let file = reader.read(&source).expect("pinned interface embodiment");
        let generated = RustEmitter::wire_contract()
            .emit(&file)
            .expect("wire contract emission");
        assert!(generated.contains("ChannelContractId"));
        assert!(generated.contains("ChannelWireRevision"));
        assert!(generated.contains("impl datomic :: Datomic"));
        compile_pinned_wire_contract(name, &generated, contract, wire);
    }
}

fn compile_pinned_wire_contract(name: &str, generated: &str, contract: u32, wire: u16) {
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-pinned-wire-contract-{name}-{}",
        std::process::id()
    ));
    let source_directory = directory.join("src");
    let test_directory = directory.join("tests");
    fs::create_dir_all(&source_directory).expect("wire contract source directory");
    fs::create_dir_all(&test_directory).expect("wire contract test directory");
    fs::write(
        directory.join("Cargo.toml"),
        wire_contract_manifest(&format!("pinned-wire-{name}")),
    )
    .expect("wire contract manifest");
    fs::write(directory.join("Cargo.lock"), fixture_lockfile()).expect("wire contract lockfile");
    fs::write(source_directory.join("generated.rs"), generated).expect("generated Signal module");
    fs::write(
        source_directory.join("lib.rs"),
        "pub mod codec;\npub mod generated;\n",
    )
    .expect("Signal module boundary");
    fs::write(
        source_directory.join("codec.rs"),
        r#"
use crate::generated::{
    Frame, FrameBody, CHANNEL_CONTRACT_ID, CHANNEL_WIRE_REVISION, INTERFACE_VERSION,
};

pub struct HandOwnedCodec;

impl HandOwnedCodec {
    pub fn envelope(body: FrameBody) -> Frame {
        Frame {
            channel_contract_id: CHANNEL_CONTRACT_ID,
            channel_wire_revision: CHANNEL_WIRE_REVISION,
            protocol_version: INTERFACE_VERSION,
            body,
        }
    }
}
"#,
    )
    .expect("hand-owned codec module");
    fs::write(
        test_directory.join("signal_boundary.rs"),
        format!(
            "use pinned_wire_{name}::{{codec::HandOwnedCodec, generated::{{ChannelContractId, ChannelWireRevision, ProtocolVersion}}}};\n\n#[test]\nfn frame_metadata_stays_generated_while_codec_is_hand_owned() {{\n    let _ = HandOwnedCodec::envelope;\n    assert_eq!(ChannelContractId({contract}).0, {contract});\n    assert_eq!(ChannelWireRevision({wire}).0, {wire});\n    assert_eq!(ProtocolVersion::new(0, 2, 0).minor, 2);\n}}\n"
        ),
    )
    .expect("wire contract boundary witness");
    assert!(
        Command::new("cargo")
            .args(["test", "--offline"])
            .env("CARGO_TARGET_DIR", directory.join("target"))
            .current_dir(&directory)
            .status()
            .expect("wire contract cargo invocation")
            .success(),
        "pinned Interface Signal module and hand-owned codec must compile together"
    );
    fs::remove_dir_all(directory).expect("wire contract cleanup");
}
