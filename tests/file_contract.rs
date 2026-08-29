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
        let generated = RustEmitter::new()
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
            if matches!(name, "FiniteDecimal" | "DatomicString") {
                item.fields = syn::Fields::Named(syn::parse_quote!({}));
            }
        }
        syn::Item::Enum(item) => item
            .attrs
            .retain(|attribute| attribute.path().is_ident("non_exhaustive")),
        syn::Item::Trait(item) => {
            item.attrs.clear();
            for trait_item in &mut item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
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
fn pinned_public_associations_compile_only_with_actual_implementations() {
    let directory = std::env::temp_dir().join(format!(
        "ethos-zero-pinned-associations-{}",
        std::process::id()
    ));
    fs::create_dir_all(directory.join("src")).expect("association harness directory");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"ethos-zero-association-witness\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nprotos = { git = \"https://github.com/LiGoldragon/protos\", rev = \"2f605fd689679a24e4ef83a643a2cd066cb12186\" }\ndatomic = { git = \"https://github.com/LiGoldragon/datomic\", rev = \"4baeaac79de265bfeb019e8dcdf7124daa9ecac5\" }\n",
    )
    .expect("association harness manifest");
    fs::write(
        directory.join("src/main.rs"),
        r#"
use datomic::{Datomic, DatomicString, DecimalViewing, FiniteDecimal};
use protos::{
    BareSafe, ContentHashable, Delineatable, DelineatedText, Embodiable, Embodied, Fault, Portion,
    PortionText, Printing, Prospective, ScalarAnatomy, Text,
};

fn text<T: Delineatable + Embodiable + ContentHashable + BareSafe + DelineatedText>() {}
fn portion<T: Printing + PortionText + ScalarAnatomy>() {}
fn datomic<T: Datomic>() {}
fn decimal<T: Datomic + DecimalViewing>() {}

struct EmbodiedValue;

impl Embodied for EmbodiedValue {
    fn from_portion(_: &Portion) -> Result<Self, Fault> { loop {} }
}

fn main() {
    text::<Text<EmbodiedValue>>();
    text::<Prospective<EmbodiedValue>>();
    portion::<Portion>();
    datomic::<DatomicString>();
    decimal::<FiniteDecimal>();
}
"#,
    )
    .expect("positive association witness");
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        directory.join("Cargo.lock"),
    )
    .expect("association harness lockfile");
    assert!(
        Command::new("cargo")
            .arg("check")
            .current_dir(&directory)
            .status()
            .expect("positive association compile")
            .success(),
        "map associations must compile against the pinned implementations"
    );
    fs::write(
        directory.join("src/main.rs"),
        "trait Required {}\nstruct Missing;\nfn need<T: Required>() {}\nfn main() { need::<Missing>(); }\n",
    )
    .expect("negative association witness");
    assert!(
        !Command::new("cargo")
            .arg("check")
            .current_dir(&directory)
            .status()
            .expect("negative association compile")
            .success(),
        "a missing required association must fail"
    );
    fs::remove_dir_all(directory).expect("association harness cleanup");
}

#[test]
fn own_and_orchestrate_interfaces_are_read_as_portion_files() {
    let reader = FileReader::new(&EmptyManifest);
    for source in [
        include_str!("../signal.ethos"),
        include_str!("../meta-signal.ethos"),
        "Interface.{0 2 0} Channel.{Orchestrate 1 5} [] {[Lock.LockRequest Release.LockId Observe.ObserveSelection] [Locked.Lock LockRejected.LockRejection Released.Lock ReleaseRejected.ReleaseRejection Observed.Observation] [] [] [LockName.String FlowId.String LockPath.String LockPaths.Vector<LockPath> LockReason.String LockRequest.{LockName FlowId LockPaths LockReason} LockId.Integer Lock.{LockId LockName FlowId LockPaths LockReason} DuplicateName.Lock LockOverlap.{LockPath Lock} LockRejection.[DuplicateName.Lock PathOverlap.LockOverlap] ReleaseRejection.[UnknownLockId] ObserveSelection.[Locks] Locks.Vector<Lock> LockSnapshot.{Locks} Observation.[Locks.LockSnapshot]]}",
        "Interface.{0 1 0} Channel.{MetaOrchestrate 2 4} [] {[Configure.Configure] [Configured.Configured ConfigurationRejected.ConfigurationRejected] [] [] [OrdinarySocketPath.String MetaSocketPath.String Configure.{OrdinarySocketPath MetaSocketPath} ConfigurationRefusal.[InvalidConfiguration] Configured.{Configure} ConfigurationRejected.{Configure ConfigurationRefusal}]}",
    ] {
        reader.read(source).expect("complete headed interface");
    }
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
        "[package]\nname = \"generated-orchestrate\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nprotos = { git = \"https://github.com/LiGoldragon/protos\", rev = \"2f605fd689679a24e4ef83a643a2cd066cb12186\" }\ndatomic = { git = \"https://github.com/LiGoldragon/datomic\", rev = \"4baeaac79de265bfeb019e8dcdf7124daa9ecac5\" }\n",
    )
    .expect("fixture manifest");
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
    fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"),
        directory.join("Cargo.lock"),
    )
    .expect("fixture lockfile");
    let target = directory.join("target");
    assert!(
        Command::new("cargo")
            .arg("test")
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
