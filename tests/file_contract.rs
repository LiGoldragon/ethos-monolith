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
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [outside:Thing outside:[One Two] outside:file.[Three] outside:dir/file.[Four] file.[Local]] {[] [] [] [] []}";
    let file = FileReader::new(&OneSourceManifest)
        .read(source)
        .expect("canonical imports");
    let ethos_zero::File::Interface(interface) = file else {
        panic!("interface");
    };
    assert_eq!(interface.imports.len(), 5);
    assert_eq!(interface.imports[0].location.file, "outside.ethos");
    assert_eq!(interface.imports[4].location.file, "file");
}

#[test]
fn vector_and_result_type_applications_are_projected_without_rust_source_assembly() {
    let source = "Interface.{0 1 0} Channel.{Example 1 0} [] {[] [] [] [] [Things.Vector<Thing> Outcome.Result<Things Reason>]}";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("application read");
    let rust = RustEmitter::new().emit(&file).expect("application emit");
    assert!(rust.contains("Vec<Thing>"));
    assert!(rust.contains("Result<Things, Reason>"));
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
fn schema_kinds_and_associations_become_traits_and_checked_impls() {
    let source =
        "Schema.{0 1 0} [] [Name.String User.{Name}] [Processable.[]] [User.[Processable]]";
    let file = FileReader::new(&EmptyManifest)
        .read(source)
        .expect("schema read");
    let rust = RustEmitter::new().emit(&file).expect("schema emit");
    assert!(rust.contains("trait Processable"));
    assert!(rust.contains("impl Processable for User"));
    syn::parse_file(&rust).expect("syntax");
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
    assert_eq!(error.extent, ethos_zero::Span { start: 0, end: 6 });
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
