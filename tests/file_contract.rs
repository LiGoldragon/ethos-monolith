use ethos_zero::{FileReader, Manifest, RustEmitter};
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
    }
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
fn datomic_schema_reaches_a_typed_projection_fault_until_executable_anatomy_exists() {
    let source = "Schema.{0 1 0} [] Types.[Fault.{Extent.String} Value.String Record.{Value} Mode.[Unit Data.Record] List.Vector<Value> Optional.Option<Value> Mapping.«Key.String Value.String»] Kinds.[Datomic.{embody.[Result<Self Fault>] portion.[Portion] textualize.[Text<Self>]}] Associations.[]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("Datomic map read");
    let error = RustEmitter::new()
        .emit(&file)
        .expect_err("Datomic algorithms are intentionally not emitted as stubs");
    assert_eq!(
        error.reason,
        ethos_zero::FileFaultReason::UnsupportedApplication
    );
}

#[test]
fn schema_kinds_and_associations_become_traits_and_checked_impls() {
    let source = "Schema.{0 1 0} [] Types.[Name.String User.{Name}] Kinds.[Processable.[]] Associations.[User.[Processable]]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("schema read");
    let rust = RustEmitter::new().emit(&file).expect("schema emit");
    assert!(rust.contains("trait Processable"));
    assert!(rust.contains("impl Processable for User"));
    syn::parse_file(&rust).expect("syntax");
}

#[test]
fn false_kind_association_fails_generated_rust_compilation() {
    let source =
        "Schema.{0 1 0} [] Types.[Name.String User.{Name}] Kinds.[] Associations.[User.[Missing]]";
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
    assert!(
        !Command::new("rustc")
            .args(["--edition", "2024", "--crate-type", "lib"])
            .arg(&source_path)
            .arg("--out-dir")
            .arg(&directory)
            .status()
            .expect("rustc invocation")
            .success()
    );
    fs::remove_dir_all(directory).expect("isolated output cleanup");
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
fn interface_file_is_read_from_the_portion_pivot_and_emitted_as_rust_syntax() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[Create.User] [Created.User] [Refused.Reason] [] [Name.String Reason.String User.{Name}]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("read file");
    let rust = RustEmitter::new().emit(&file).expect("emit Rust");
    syn::parse_file(rust.as_ref()).expect("generated Rust parses");
    let directory =
        std::env::temp_dir().join(format!("ethos-zero-contract-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("isolated output directory");
    let source_path = directory.join("generated.rs");
    fs::write(&source_path, rust).expect("generated Rust source");
    assert!(
        Command::new("rustc")
            .args(["--edition", "2024", "--crate-type", "lib"])
            .arg(&source_path)
            .arg("--out-dir")
            .arg(&directory)
            .status()
            .expect("rustc invocation")
            .success()
    );
    fs::remove_dir_all(directory).expect("isolated output cleanup");
}
