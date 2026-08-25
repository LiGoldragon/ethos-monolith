use ethos_monolith::fixture::{
    Channel, EnumVariantElement, INTERFACE_SOURCE, InputElement, InterfaceEvidencedRealizing,
    InterfaceEvidencedTextualizing, InterfaceText, NamedEnum, NamedStruct, NamedTypedef,
    OutputElement, RefusalElement, SignalArtifactProjecting, StreamElement, TypeElement, Version,
};
use protos::{
    FrameObserving, ObservationViewing, ParentObserving, Realize, Textualize, TransitionObserving,
    WalkTransitionKind,
};

#[test]
fn fixture_realizes_and_projects_with_walk_evidence() {
    let text = InterfaceText {
        source: protos::SourceText(INTERFACE_SOURCE.into()),
    };
    let realized = text.realize().expect("fixture realizes");
    assert!(
        realized.imports.0.is_empty(),
        "the psyche fixture contract has no imports"
    );
    assert_eq!(
        realized.version,
        Version {
            major: 0,
            minor: 1,
            patch: 0,
        }
    );
    assert_eq!(
        realized.channel,
        Channel {
            name: "Psyche".into(),
            contract_id: 1,
            wire_revision: 1,
        }
    );
    assert_eq!(
        realized.inputs.0,
        vec![
            InputElement {
                operation: "Record".into(),
                payload: "Entry".into(),
            },
            InputElement {
                operation: "Subscribe".into(),
                payload: "Layer".into(),
            },
            InputElement {
                operation: "Unsubscribe".into(),
                payload: "SubscriptionHandle".into(),
            },
        ]
    );
    assert_eq!(
        realized.outputs.0,
        vec![
            OutputElement {
                operation: "Recorded".into(),
                payload: "EntryIdentifier".into(),
            },
            OutputElement {
                operation: "Subscribed".into(),
                payload: "SubscriptionHandle".into(),
            },
            OutputElement {
                operation: "Unsubscribed".into(),
                payload: "SubscriptionHandle".into(),
            },
        ]
    );
    assert_eq!(
        realized.refusals.0,
        vec![
            RefusalElement {
                operation: "AdmissionRejected".into(),
                payload: "AdmissionRefusal".into(),
            },
            RefusalElement {
                operation: "UnknownSubscription".into(),
                payload: "SubscriptionHandle".into(),
            },
        ]
    );
    assert_eq!(
        realized.streams.0,
        vec![StreamElement {
            operation: "RecordChange".into(),
            payload: "RecordEvent".into(),
        }]
    );
    assert_eq!(
        realized.types.0,
        vec![
            TypeElement::Enum(NamedEnum {
                name: "Layer".into(),
                variants: vec![
                    EnumVariantElement::Unit {
                        variant: "Spirit".into(),
                    },
                    EnumVariantElement::Unit {
                        variant: "Intent".into(),
                    },
                    EnumVariantElement::Unit {
                        variant: "Vision".into(),
                    },
                ],
            }),
            TypeElement::Typedef(NamedTypedef {
                name: "Description".into(),
                target: "String".into(),
            }),
            TypeElement::Typedef(NamedTypedef {
                name: "EntryIdentifier".into(),
                target: "String".into(),
            }),
            TypeElement::Typedef(NamedTypedef {
                name: "SubscriptionHandle".into(),
                target: "Integer".into(),
            }),
            TypeElement::Struct(NamedStruct {
                name: "Entry".into(),
                fields: vec!["Layer".into(), "Description".into()],
            }),
            TypeElement::Enum(NamedEnum {
                name: "AdmissionRefusal".into(),
                variants: vec![
                    EnumVariantElement::Unit {
                        variant: "Duplicate".into(),
                    },
                    EnumVariantElement::Unit {
                        variant: "InvalidRequest".into(),
                    },
                ],
            }),
            TypeElement::Enum(NamedEnum {
                name: "RecordEvent".into(),
                variants: vec![EnumVariantElement::Data {
                    variant: "RecordRecorded".into(),
                    payload: "EntryIdentifier".into(),
                }],
            }),
        ]
    );
    let projected = realized.textualize().expect("fixture projects");
    let round_trip = projected.realize().expect("projection realizes");
    assert_eq!(realized, round_trip);
}

fn assert_walk_evidence(evidence: &ethos_monolith::fixture::InterfaceEvidence) {
    let history = evidence.observation.history();
    assert!(
        history
            .iter()
            .any(|transition| transition.kind() == WalkTransitionKind::Enter)
    );
    assert!(
        history
            .iter()
            .any(|transition| transition.kind() == WalkTransitionKind::Close)
    );
    assert_eq!(
        history
            .iter()
            .filter(|transition| transition.kind() == WalkTransitionKind::Resume)
            .count(),
        evidence.observation.resumptions()
    );
    assert!(history.windows(2).any(|window| {
        let [closed, resumed] = window else {
            return false;
        };
        if closed.kind() != WalkTransitionKind::Close
            || resumed.kind() != WalkTransitionKind::Resume
        {
            return false;
        }
        let Some(before) = resumed.parent_before() else {
            return false;
        };
        let Some(after) = resumed.parent_after() else {
            return false;
        };
        before.frame() == after.frame()
            && after.position() == before.position() + 1
            && closed.frame().identity() != resumed.frame().identity()
    }));
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
    assert_walk_evidence(&realized.evidence);

    let projected = realized
        .value
        .textualize_evidenced()
        .expect("fixture textualizes with evidence");
    assert_eq!(projected.evidence.cursor, projected.text.source.0.len());
    assert_eq!(projected.evidence.observation.depth(), 0);
    assert!(projected.evidence.observation.resumptions() >= 10);
    assert_walk_evidence(&projected.evidence);
}

#[test]
fn fixture_projects_a_signal_module() {
    let text = InterfaceText {
        source: protos::SourceText(INTERFACE_SOURCE.into()),
    };
    let interface = text.realize().expect("fixture realizes");
    let generated = interface.signal_rust_source().expect("projection succeeds");

    assert!(generated.contains("pub enum PsycheWire {}"));
    assert!(generated.contains("signal_channel!"));
    syn::parse_file(&generated).expect("signal projection is Rust");
}

#[test]
fn malformed_fixture_is_rejected_without_a_fake_tree() {
    for source in [
        "Interface.{0 1 0} Channel.{Fixture 1 1} [] {[Bad.lower] [] [] [] []}",
        "Interface.{0 1 0} Channel.{Fixture 1 1} [] {[Record.Entry<Integer>] [] [] [] []}",
        "Interface.{0 1 0} Channel.{Fixture 1 1} [] {[Record.{Entry}] [] [] [] []}",
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
