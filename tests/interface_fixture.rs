use std::path::PathBuf;

use ethos_monolith::fixture::{
    EnumVariantElement, INTERFACE_SOURCE, InterfaceEvidencedRealizing, InterfaceText,
    RustArtifactProjecting, TypeElement, generated,
};
use protos::{ObservationViewing, Realize, Textualize};

#[test]
fn psyche_fixture_is_a_real_consumer_of_generated_types() {
    let _ = generated::Input::Record(generated::Entry {
        layer: generated::Layer::Spirit,
        description: generated::Description("overnight".into()),
    });
}

#[test]
fn fixture_realizes_and_projects_with_walk_evidence() {
    let text = InterfaceText {
        source: protos::SourceText(INTERFACE_SOURCE.into()),
    };
    let realized = text.realize().expect("fixture realizes");
    assert_eq!(realized.version.major, 0);
    assert_eq!(realized.inputs.0[0].operation, "Record");
    assert!(matches!(
        &realized.types.0[0],
        TypeElement::Enum(value)
            if matches!(value.variants[0], EnumVariantElement::Unit { .. })
    ));
    assert!(matches!(
        &realized.types.0[6],
        TypeElement::Enum(value)
            if matches!(value.variants[0], EnumVariantElement::Data { .. })
    ));
    let projected = realized.textualize().expect("fixture projects");
    let round_trip = projected.realize().expect("projection realizes");
    assert_eq!(realized, round_trip);
}

#[test]
fn fixture_walks_nested_sections_and_resumes_parents() {
    let text = InterfaceText {
        source: protos::SourceText(INTERFACE_SOURCE.into()),
    };
    let realized = text
        .realize_evidenced()
        .expect("fixture realizes with evidence");
    assert_eq!(realized.evidence.cursor, INTERFACE_SOURCE.len());
    assert_eq!(realized.evidence.observation.depth(), 0);
    assert!(realized.evidence.observation.resumptions() >= 10);
}

#[test]
fn committed_generated_fixture_matches_projection() {
    let text = InterfaceText {
        source: protos::SourceText(INTERFACE_SOURCE.into()),
    };
    let interface = text.realize().expect("fixture realizes");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/fixture/generated.rs");
    interface
        .rust_artifact(path)
        .expect("projection succeeds")
        .assert_matches_existing()
        .expect("committed generated fixture is fresh");
}

#[test]
fn malformed_fixture_is_rejected_without_a_fake_tree() {
    for source in [
        "Interface.{0 1 0} [] {[Bad.lower] [] [] [] []}",
        "Interface.{0 1 0} [] {[Record.Entry<Integer>] [] [] [] []}",
        "Interface.{0 1 0} [] {[Record.{Entry}] [] [] [] []}",
    ] {
        let text = InterfaceText {
            source: protos::SourceText(source.into()),
        };
        assert!(
            text.realize().is_err(),
            "source should be rejected: {source}"
        );
    }
}
