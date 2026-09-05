//! The CLI speaks its contract: one inline datom value, no flags, every
//! reply a textualized Response.

use std::process::Command;

/// The kind whose capability runs the built binary with these arguments.
trait Invoking {
    fn invoke(&self) -> (bool, String);
}

impl Invoking for [&str] {
    fn invoke(&self) -> (bool, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_ethos-zero"))
            .args(self)
            .output()
            .expect("the binary runs");
        (
            output.status.success(),
            String::from_utf8(output.stdout).expect("utf-8 output"),
        )
    }
}

/// The kind whose capability yields a fresh directory under the target directory.
trait Scratching {
    fn scratch(&self) -> String;
}

impl Scratching for str {
    fn scratch(&self) -> String {
        let directory = format!(
            "{}/cli-{}-{}",
            env!("CARGO_TARGET_TMPDIR"),
            self,
            std::process::id()
        );
        let _ = std::fs::remove_dir_all(&directory);
        directory
    }
}

#[test]
fn no_argument_prints_the_crates_own_ethos_ending_with_a_newline() {
    let (success, output) = [].invoke();
    assert!(success);
    assert_eq!(output, include_str!("../ethos-zero.ethos"));
    assert!(output.ends_with('\n'));
    assert!(output.contains("[ Generate.Generation ]"));
}

#[test]
fn generate_writes_the_stem_and_replies_generated() {
    let directory = "generate".scratch();
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/record-types.ethos");
    let (success, output) = [format!("Generate.{{ {source} {directory} }}").as_str()].invoke();
    assert!(success, "{output}");
    assert_eq!(
        output,
        format!("Generated.[ {directory}/record-types.rs ]\n")
    );
    let written = std::fs::read_to_string(format!("{directory}/record-types.rs")).unwrap();
    assert!(written.contains("pub struct Record(pub protos::Text, pub protos::Integer);"));
}

#[test]
fn a_quoted_path_is_a_path() {
    let directory = "quoted".scratch();
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/multi-types.ethos");
    let (success, output) =
        [format!("Generate.{{ \u{201C}{source}\u{201D} \u{201C}{directory}\u{201D} }}").as_str()]
            .invoke();
    assert!(success, "{output}");
    assert_eq!(
        output,
        format!("Generated.[ {directory}/multi-types.rs ]\n")
    );
}

#[test]
fn a_flag_and_a_wrong_shape_are_refused_as_malformed() {
    let (success, output) = ["--help"].invoke();
    assert!(!success);
    assert_eq!(
        output,
        "Malformed.{ None Corporate.{ [] Shape.{ Variant --help } } }\n"
    );
    let (success, output) = ["Generate.{ /only }"].invoke();
    assert!(!success);
    assert!(output.starts_with("Malformed.{ "), "{output}");
    assert!(output.contains("Arity.{ 2 1 }"), "{output}");
    let (success, output) = ["Bogus.{ /a /b }"].invoke();
    assert!(!success);
    assert_eq!(
        output,
        "Malformed.{ None Corporate.{ [] UnknownVariant.Bogus } }\n"
    );
}

#[test]
fn more_than_one_argument_is_refused_with_the_count() {
    let (success, output) = ["Generate.{", "/a", "/b", "}"].invoke();
    assert!(!success);
    assert_eq!(output, "Arguments.4\n");
}

#[test]
fn a_missing_file_is_unreadable() {
    let (success, output) = ["Generate.{ /nowhere/missing.ethos /nowhere }"].invoke();
    assert!(!success);
    assert!(
        output.starts_with("Unreadable.{ /nowhere/missing.ethos "),
        "{output}"
    );
}

#[test]
fn a_faulty_file_replies_the_situated_fault() {
    let directory = "faulty".scratch();
    std::fs::create_dir_all(&directory).unwrap();
    let source = format!("{directory}/faulty.ethos");
    std::fs::write(&source, "Types\n[]\n[ Record.{ Text Bogus } ]\n[]").unwrap();
    let (success, output) = [format!("Generate.{{ {source} {directory} }}").as_str()].invoke();
    assert!(!success);
    assert_eq!(
        output,
        format!(
            "Faulty.{{ {source} {{ Some.{{ 25 30 }} Conceptual.{{ [ 0 0 1 0 0 1 ] Undeclared.Bogus }} }} }}\n"
        )
    );
}
