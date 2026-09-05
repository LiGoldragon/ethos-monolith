//! Freshness: every committed generated module equals a fresh generation
//! by the library. The text is the product here, so the text is what is
//! asserted: src/fault.rs from fault.ethos, src/contract.rs from
//! ethos-zero.ethos, and tests/generated/<stem>.rs from every fixture.

use ethos_zero::{File, Generating};
use protos::{Actualizable, Potential};

/// The kind whose capability asserts a committed generation is fresh.
trait Fresh {
    fn fresh(&self, generated: &str);
}

impl Fresh for str {
    fn fresh(&self, generated: &str) {
        let root = env!("CARGO_MANIFEST_DIR");
        let source = std::fs::read_to_string(format!("{root}/{self}")).expect(self);
        let committed = std::fs::read_to_string(format!("{root}/{generated}")).expect(generated);
        let file = match Potential::<File>::from(source).actualize(()) {
            Ok(file) => file,
            Err(fault) => panic!("{self} does not read: {fault:?}"),
        };
        assert_eq!(
            file.generate(),
            committed,
            "{generated} is stale; regenerate from {self}"
        );
    }
}

#[test]
fn the_fault_module_is_fresh() {
    "fault.ethos".fresh("src/fault.rs");
}

#[test]
fn the_contract_module_is_fresh() {
    "ethos-zero.ethos".fresh("src/contract.rs");
}

#[test]
fn every_fixture_module_is_fresh() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut fixtures: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(format!("{root}/fixtures")).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if let Some(stem) = name.strip_suffix(".ethos") {
            fixtures.push(stem.to_owned());
        }
    }
    fixtures.sort();
    assert_eq!(fixtures.len(), 10);
    for stem in fixtures {
        format!("fixtures/{stem}.ethos").fresh(&format!("tests/generated/{stem}.rs"));
    }
}
