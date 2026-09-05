//! The reader and the generator on every shape of Vision/ethos.md and
//! every adversarial input of the audit. A fault is asserted with its
//! path and, where the delineation situates it, its extent.

use ethos_zero::{
    Canonicalizable, Constraint, Fault, File, Form, Generating, Identity, Import, KindBody,
    Problem, Receiver, Reference, Signature, TypeDeclaration, Variant,
};
use protos::{Actualizable, Extent, Potential, Protosizable, Situated, Situation, Textualizable};

// ---------------------------------------------------------------------------
// Reading helpers, as kinds on the text
// ---------------------------------------------------------------------------

/// The kind whose capabilities read an ethos text, or fault on it.
trait Reading {
    fn file(&self) -> File;
    fn fault(&self) -> Situated<Fault>;
    fn rust(&self) -> String;
}

impl Reading for str {
    fn file(&self) -> File {
        match Potential::<File>::from(self).actualize() {
            Ok(file) => file,
            Err(fault) => panic!("{self}\ndoes not read: {fault:?}"),
        }
    }

    fn fault(&self) -> Situated<Fault> {
        match Potential::<File>::from(self).actualize() {
            Ok(file) => panic!("{self}\nwas expected to fault, read as {file:?}"),
            Err(fault) => fault,
        }
    }

    fn rust(&self) -> String {
        self.file().generate()
    }
}

/// The kind whose capability yields the conceptual problem and path of a situated fault.
trait Problematic {
    fn problem(&self) -> (Vec<i64>, Problem);
}

impl Problematic for Situated<Fault> {
    fn problem(&self) -> (Vec<i64>, Problem) {
        match &self.1 {
            Fault::Conceptual(path, problem) => (path.clone(), problem.clone()),
            Fault::Structural(fault) => panic!("structural fault, not conceptual: {fault:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Canonicalization: the sweet form opened, faithfully situated
// ---------------------------------------------------------------------------

#[test]
fn sweet_form_opens_into_the_braced_form() {
    let canonical = "Types\n[]\n[ Record.{ Text } ]\n[]"
        .to_owned()
        .canonicalize()
        .unwrap();
    assert_eq!(canonical.text, "Types.{\n[]\n[ Record.{ Text } ]\n[]\n}");
    assert_eq!(canonical.seam, Extent(5, 7));
}

#[test]
fn braced_form_is_left_as_it_is() {
    let source = "Types.{ [] [ Record.{ Text } ] [] }";
    let canonical = source.to_owned().canonicalize().unwrap();
    assert_eq!(canonical.text, source);
    assert_eq!(canonical.seam, Extent(0, 0));
    assert_eq!(source.file(), "Types\n[]\n[ Record.{ Text } ]\n[]".file());
}

#[test]
fn a_comment_on_the_head_line_reads() {
    let file = "Types ; the head\n[]\n[ Record.{ Text } ]\n[]".file();
    assert!(matches!(file, File::Types(_)));
}

#[test]
fn a_trailing_comment_without_a_final_newline_reads() {
    let file = "Types\n[]\n[ Record.{ Text } ]\n[] ; trailing".file();
    assert!(matches!(file, File::Types(_)));
}

#[test]
fn a_leading_comment_before_the_head_reads() {
    let file = "; about\n; more\nTypes\n[]\n[]\n[]".file();
    assert!(matches!(file, File::Types(_)));
}

#[test]
fn faults_are_situated_in_the_source_text() {
    let source = "Types\n[]\n[ Record.{ Text Bogus } ]\n[]";
    let fault = source.fault();
    let Situated(
        protos::Situation {
            extent: Extent(start, end),
            ..
        },
        _,
    ) = &fault;
    assert_eq!(&source[*start as usize..*end as usize], "Bogus");
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 1],
            Problem::Undeclared(protos::Text::try_from("Bogus").unwrap())
        )
    );
}

// ---------------------------------------------------------------------------
// The four variants
// ---------------------------------------------------------------------------

#[test]
fn types_reads_its_three_sections() {
    let File::Types(types) = "Types\n[ protos:Text ]\n[ Record.{ Text Integer } SinkError.[ Closed Full ] LockId.Integer ]\n[]".file() else {
        panic!("expected Types");
    };
    assert_eq!(
        types.imports,
        vec![Import::One(
            ethos_zero::Source("protos".to_owned()),
            ethos_zero::Imported {
                name: ethos_zero::Name("Text".to_owned()),
                emitted: ethos_zero::Name("Text".to_owned()),
            }
        )]
    );
    assert_eq!(types.types.len(), 3);
    assert!(
        matches!(&types.types[0], TypeDeclaration::Struct(identity, positions) if identity.name.0 == "Record" && positions.len() == 2)
    );
    assert!(
        matches!(&types.types[1], TypeDeclaration::Enum(_, variants) if variants == &[Variant::Bare(ethos_zero::Name("Closed".to_owned())), Variant::Bare(ethos_zero::Name("Full".to_owned()))])
    );
    assert!(
        matches!(&types.types[2], TypeDeclaration::Alias(_, aliased) if aliased.name.0 == "Integer")
    );
}

#[test]
fn kinds_reads_simple_and_complex_kinds() {
    let File::Kinds(kinds) = "Kinds\n[ super:[ Fillable Serializable ] ]\n[ Summarizable.[ summarize.[ Text ] ] Streamable.{ [ Fillable ] [ Item<Serializable> ] [ CAPACITY.Integer ] [ next![ Option<Item> ] ] } ]".file() else {
        panic!("expected Kinds");
    };
    let KindBody::Simple(capabilities) = &kinds.kinds[0].body else {
        panic!("expected a simple kind");
    };
    assert_eq!(capabilities[0].receiver, Receiver::Shared);
    assert!(
        matches!(&capabilities[0].signature, Signature::Yielding(yields) if yields.name.0 == "Text")
    );
    let KindBody::Complex {
        superkinds,
        types,
        constants,
        capabilities,
    } = &kinds.kinds[1].body
    else {
        panic!("expected a complex kind");
    };
    assert_eq!(superkinds[0].name.0, "Fillable");
    assert_eq!(types[0].name.0, "Item");
    assert_eq!(types[0].bounds[0].name.0, "Serializable");
    assert_eq!(constants[0].name.0, "CAPACITY");
    assert_eq!(capabilities[0].receiver, Receiver::Mutable);
}

#[test]
fn kind_identity_carries_its_constraints() {
    let File::Kinds(kinds) = "Kinds\n[ super:Clonable super:Sendable super:Serializable ]\n[ Processable<[Clonable Sendable] Serializable>.[ process.[ Text ] ] ]".file() else {
        panic!("expected Kinds");
    };
    let identity = &kinds.kinds[0].identity;
    assert_eq!(identity.name.0, "Processable");
    assert!(matches!(&identity.constraints[0], Constraint::Many(bounds) if bounds.len() == 2));
    assert!(
        matches!(&identity.constraints[1], Constraint::One(bound) if bound.name.0 == "Serializable")
    );
}

#[test]
fn signal_reads_requests_responses_and_types() {
    let File::Signal(signal) = "Signal\n[]\n[ Lock.LockRequest Release.LockId ]\n[ Locked.Lock Released.Lock ]\n[ LockId.Integer LockRequest.{ Text Text } Lock.{ Integer Text Text } ]".file() else {
        panic!("expected Signal");
    };
    assert_eq!(signal.requests.len(), 2);
    assert_eq!(signal.responses.len(), 2);
    assert_eq!(signal.types.len(), 3);
}

#[test]
fn sema_reads_the_record_positions_and_types() {
    let File::Sema(sema) = "Sema\n[]\n{ Text Vector<Entry> }\n[ Entry.{ Text Integer } ]".file()
    else {
        panic!("expected Sema");
    };
    assert_eq!(sema.record.len(), 2);
    assert_eq!(sema.types.len(), 1);
}

#[test]
fn capability_receivers_follow_the_separator() {
    let File::Kinds(kinds) =
        "Kinds\n[]\n[ Test.[ read.[ Text ] write![ Text ] create:[ Self ] ] ]".file()
    else {
        panic!("expected Kinds");
    };
    let KindBody::Simple(capabilities) = &kinds.kinds[0].body else {
        panic!("expected a simple kind");
    };
    assert_eq!(capabilities[0].receiver, Receiver::Shared);
    assert_eq!(capabilities[1].receiver, Receiver::Mutable);
    assert_eq!(capabilities[2].receiver, Receiver::Static);
}

#[test]
fn inline_qualification_reads_as_a_sourced_reference() {
    let File::Types(types) =
        "Types\n[]\n[ Fault.[ Structural.protos:Fault Own.Problem ] Problem.[ Root ] ]\n[]".file()
    else {
        panic!("expected Types");
    };
    let TypeDeclaration::Enum(_, variants) = &types.types[0] else {
        panic!("expected an enum");
    };
    assert_eq!(
        variants[0],
        Variant::Typed(
            ethos_zero::Name("Structural".to_owned()),
            Reference {
                source: Some(ethos_zero::Source("protos".to_owned())),
                name: ethos_zero::Name("Fault".to_owned()),
                arguments: vec![],
            }
        )
    );
}

// ---------------------------------------------------------------------------
// Generated Rust: the vision's examples, line for line
// ---------------------------------------------------------------------------

#[test]
fn generates_the_vision_types() {
    let rust = "Types\n[]\n[ Record.{ Text Integer } Report.{ Text Vector<Integer> } SinkError.[ Closed Full ] LockId.Integer ]\n[]".rust();
    assert!(rust.contains("pub struct Record(pub protos::Text, pub protos::Integer);"));
    assert!(rust.contains("pub struct Report(pub protos::Text, pub Vec<protos::Integer>);"));
    assert!(rust.contains("pub enum SinkError {\n    Closed,\n    Full,\n}"));
    assert!(rust.contains("pub type LockId = protos::Integer;"));
    assert!(!rust.contains("use "));
}

#[test]
fn generates_the_vision_kinds() {
    let rust = "Kinds\n[ super:SinkError ]\n[ Summarizable.[ summarize.[ Text ] ] Fillable.[ push!{ [ Text ] [ Result<Integer SinkError> ] } drain![ Vector<Text> ] create:[ Self ] ] ]".rust();
    assert!(rust.contains("pub trait Summarizable {\n    fn summarize(&self) -> protos::Text;\n}"));
    assert!(rust.contains(
        "fn push(&mut self, input: protos::Text) -> Result<protos::Integer, super::SinkError>;"
    ));
    assert!(rust.contains("fn drain(&mut self) -> Vec<protos::Text>;"));
    assert!(rust.contains("fn create() -> Self;"));
}

#[test]
fn generates_the_streamable_kind_with_self_qualified_associated_types() {
    let rust = "Kinds\n[ super:[ Fillable Serializable ] ]\n[ Streamable.{ [ Fillable ] [ Item<Serializable> ] [ CAPACITY.Integer ] [ next![ Option<Item> ] ] } ]".rust();
    assert!(rust.contains("pub trait Streamable: super::Fillable {"));
    assert!(rust.contains("type Item: super::Serializable;"));
    assert!(rust.contains("const CAPACITY: protos::Integer;"));
    assert!(rust.contains("fn next(&mut self) -> Option<Self::Item>;"));
}

#[test]
fn generates_the_identity_with_the_sources_names() {
    let rust = "Kinds\n[ super:Clonable super:Sendable super:Serializable ]\n[ Processable<[Clonable Sendable] Serializable>.[ process.[ Text ] ] ]".rust();
    assert!(rust.contains(
        "pub trait Processable<A: super::Clonable + super::Sendable, B: super::Serializable> {"
    ));
}

#[test]
fn generates_the_vision_association() {
    let rust = "Types\n[ super:[ Summarizable Fillable ] ]\n[ Sink.{ Text } ]\n[ Sink.[ Summarizable Fillable ] ]".rust();
    assert!(rust.contains("const _: () = {\n    fn assert_sink_summarizable<T: super::Summarizable>() {}\n    let _ = assert_sink_summarizable::<Sink>;\n    fn assert_sink_fillable<T: super::Fillable>() {}\n    let _ = assert_sink_fillable::<Sink>;\n};"));
}

#[test]
fn generates_a_constrained_association_as_a_generic_assertion() {
    let rust = "Types\n[ datomic:[ Datomic Situated ] ]\n[ Own.{ Text } ]\n[ Situated<Datomic>.[ Datomic ] Text.[ Datomic ] ]".rust();
    assert!(rust.contains("fn assert_situated_datomic<A: datomic::Datomic>() {\n        fn assertion<T: datomic::Datomic>() {}\n        let _ = assertion::<datomic::Situated<A>>;\n    }"));
    assert!(rust.contains("let _ = assert_text_datomic::<protos::Text>;"));
}

#[test]
fn generates_the_vision_request_enum() {
    let rust = "Signal\n[]\n[ Lock.LockRequest Release.LockId Observe.ObserveSelection ]\n[ Done ]\n[ LockRequest.{ Text } LockId.Integer ObserveSelection.[ Locks ] ]".rust();
    assert!(rust.contains("pub enum Request {\n    Lock(LockRequest),\n    Release(LockId),\n    Observe(ObserveSelection),\n}"));
    assert!(rust.contains("pub enum Response {\n    Done,\n}"));
}

#[test]
fn generates_tuple_variants_and_boxes_only_where_rust_needs_it() {
    let rust = "Types\n[]\n[ Tree.[ Leaf.Integer Node.{ Tree Tree } Many.Vector<Tree> Maybe.Option<Tree> ] Chain.{ Text Option<Chain> } ]\n[]".rust();
    assert!(rust.contains("Node(Box<Tree>, Box<Tree>)"));
    assert!(rust.contains("Many(Vec<Tree>)"));
    assert!(rust.contains("Maybe(Box<Option<Tree>>)"));
    assert!(rust.contains("pub struct Chain(pub protos::Text, pub Box<Option<Chain>>);"));
    assert!(!rust.contains("TreeNode"));
}

#[test]
fn generates_a_constrained_type_with_its_parameter() {
    let rust = "Types\n[]\n[ Placed<Sized>.{ Option<Integer> Sized } ]\n[]".rust();
    assert!(rust.contains("pub struct Placed<A: Sized>(pub Option<protos::Integer>, pub A);"));
    assert!(
        rust.contains("impl<A: Sized + datom_codec::Datomic> datom_codec::Datomic for Placed<A> {")
    );
}

#[test]
fn generates_eq_unless_decimal_is_reached() {
    let rust =
        "Types\n[]\n[ Score.{ Decimal } Wrapped.{ Option<Score> } Plain.{ Integer } ]\n[]".rust();
    assert!(rust.contains("#[derive(Clone, Debug, PartialEq)]\npub struct Score"));
    assert!(rust.contains("#[derive(Clone, Debug, PartialEq)]\npub struct Wrapped"));
    assert!(rust.contains("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct Plain"));
}

#[test]
fn generates_the_position_index_into_every_fault_path() {
    let rust = "Types\n[]\n[ Record.{ Text Integer } ]\n[]".rust();
    assert!(rust.contains("datom_codec::Positional::position"));
    assert!(rust.contains("datom_codec::Sited::positions(site, 2)"));
}

#[test]
fn generates_a_constant_of_a_container_type() {
    let rust = "Kinds\n[]\n[ Naming.{ [] [] [ NAMES.Vector<Text> ] [ name.[ Text ] ] } ]".rust();
    assert!(rust.contains("const NAMES: Vec<protos::Text>;"));
}

// ---------------------------------------------------------------------------
// The ascent: File -> Text -> File
// ---------------------------------------------------------------------------

#[test]
fn every_variant_round_trips_through_its_canonical_text() {
    for source in [
        "Types\n[ protos:Text ]\n[ Record.{ Text Integer } ]\n[]",
        "Kinds\n[ super:SinkError ]\n[ Fillable.[ push!{ [ Text ] [ Result<Integer SinkError> ] } drain![ Vector<Text> ] ] Streamable.{ [ Fillable ] [ Item<Fillable> ] [ CAPACITY.Integer ] [ next![ Option<Item> ] ] } Processable<[Fillable Streamable] Fillable>.[ process.[ Text ] ] ]",
        "Signal\n[ ethos_zero:Fault ]\n[ Lock.LockRequest ]\n[ Locked.Lock Faulty.{ Text Fault } Structural.protos:Fault ]\n[ LockRequest.{ Text } Lock.{ Integer Text } ]",
        "Sema\n[]\n{ Text Vector<Entry> }\n[ Entry.{ Text Integer } ]",
    ] {
        let file = source.file();
        let text = file.textualize();
        let again: File = Potential::<File>::from(text.clone()).actualize().unwrap();
        assert_eq!(file, again, "{text}");
        let delineation = Protosizable::protosize(&file).unwrap();
        assert_eq!(delineation.textualize(), text);
    }
}

// ---------------------------------------------------------------------------
// Adversarial inputs: each a typed, situated fault; none aborts
// ---------------------------------------------------------------------------

#[test]
fn rejects_an_undeclared_name_in_a_type_position() {
    let fault = "Types\n[]\n[ Record.{ Text Bogus } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 1],
            Problem::Undeclared(protos::Text::try_from("Bogus").unwrap())
        )
    );
}

#[test]
fn rejects_an_undeclared_superkind_and_bound_and_constraint() {
    let fault = "Kinds\n[]\n[ Streamable.{ [ Fillable ] [] [] [ next![ Text ] ] } ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 0, 0],
            Problem::Undeclared(protos::Text::try_from("Fillable").unwrap())
        )
    );
    let fault =
        "Kinds\n[]\n[ Streamable.{ [] [ Item<Serializable> ] [] [ next![ Text ] ] } ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 1, 0, 0],
            Problem::Undeclared(protos::Text::try_from("Serializable").unwrap())
        )
    );
    let fault = "Kinds\n[]\n[ Processable<Clonable>.[ process.[ Text ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0],
            Problem::Undeclared(protos::Text::try_from("Clonable").unwrap())
        )
    );
}

#[test]
fn rejects_an_association_to_an_undeclared_kind_or_type() {
    let fault = "Types\n[]\n[ Sink.{ Text } ]\n[ Sink.[ Summarizable ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 2, 0, 0, 0],
            Problem::Undeclared(protos::Text::try_from("Summarizable").unwrap())
        )
    );
    let fault = "Types\n[ super:Summarizable ]\n[]\n[ Sink.[ Summarizable ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 2, 0],
            Problem::Undeclared(protos::Text::try_from("Sink").unwrap())
        )
    );
}

#[test]
fn rejects_a_type_where_a_kind_is_asked_and_the_reverse() {
    let fault =
        "Types\n[ datom_codec:Datomic ]\n[ Sink.{ Text } ]\n[ Vector<Sink>.[ Datomic ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 2, 0, 0],
            Problem::Role(protos::Text::try_from("Sink").unwrap())
        )
    );
    let fault = "Kinds\n[]\n[ Own.[ own.[ Sized ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 0, 0, 0],
            Problem::Role(protos::Text::try_from("Sized").unwrap())
        )
    );
}

#[test]
fn rejects_a_duplicate_type_kind_variant_or_import() {
    let fault = "Types\n[]\n[ Record.{ Text } Record.{ Integer } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 1],
            Problem::Duplicate(protos::Text::try_from("Record").unwrap())
        )
    );
    let fault = "Kinds\n[]\n[ Own.[ own.[ Text ] ] Own.[ own.[ Text ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 1],
            Problem::Duplicate(protos::Text::try_from("Own").unwrap())
        )
    );
    let fault = "Types\n[]\n[ Twice.[ A B A ] ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 2],
            Problem::Duplicate(protos::Text::try_from("A").unwrap())
        )
    );
    let fault = "Types\n[ protos:Text ]\n[ Text.{ Integer } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0],
            Problem::Duplicate(protos::Text::try_from("Text").unwrap())
        )
    );
    let fault = "Signal\n[]\n[ Go ]\n[ Done ]\n[ Request.{ Text } ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 3, 0],
            Problem::Duplicate(protos::Text::try_from("Request").unwrap())
        )
    );
}

#[test]
fn rejects_a_capability_without_a_yield() {
    let fault = "Kinds\n[]\n[ Own.[ run.[] ] ]".fault();
    assert_eq!(fault.problem(), (vec![0, 0, 1, 0, 0, 0, 0], Problem::Yield));
    let fault = "Kinds\n[]\n[ Own.[ run.{ [ Text ] [] } ] ]".fault();
    assert_eq!(
        fault.problem(),
        (vec![0, 0, 1, 0, 0, 0, 0, 1], Problem::Yield)
    );
    let fault = "Kinds\n[]\n[ Own.[ run.[ Text Text ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (vec![0, 0, 1, 0, 0, 0, 0], Problem::Arity(1, 2))
    );
}

#[test]
fn rejects_a_signal_with_an_empty_requests_or_responses_section() {
    let fault = "Signal\n[]\n[]\n[ Done ]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0, 0, 1], Problem::Empty));
    let fault = "Signal\n[]\n[ Go ]\n[]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0, 0, 2], Problem::Empty));
}

#[test]
fn rejects_a_head_not_among_the_four() {
    let fault = "Library\n[]\n[]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0], Problem::Root));
    let fault = "Library.{ 0 1 0 }\n[]\n[]\n[]\n[]".fault();
    assert_eq!(fault.problem(), (vec![], Problem::Root));
    let fault = "".fault();
    assert_eq!(fault.problem(), (vec![], Problem::Root));
    let fault = "Types.[ [] [] [] ]".fault();
    assert_eq!(fault.problem(), (vec![0, 0], Problem::Expected(Form::File)));
}

#[test]
fn rejects_a_wrong_section_count() {
    let fault = "Types\n[]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0, 0], Problem::Arity(3, 2)));
    let fault = "Kinds\n[]\n[]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0, 0], Problem::Arity(2, 3)));
}

#[test]
fn rejects_a_name_that_is_not_an_identifier_without_panicking() {
    let fault = "Types\n[]\n[ weird-name.{ Text } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0],
            Problem::Name(protos::Text::try_from("weird-name").unwrap())
        )
    );
    let fault = "Kinds\n[]\n[ Own.[ match.[ Text ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0, 0],
            Problem::Name(protos::Text::try_from("match").unwrap())
        )
    );
    let fault = "Types\n[ 1bad:Text ]\n[]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 0, 0],
            Problem::Name(protos::Text::try_from("1bad").unwrap())
        )
    );
}

#[test]
fn rejects_a_kind_declared_with_the_wrong_separator() {
    let fault = "Kinds\n[]\n[ Summarizable:[ summarize.[ Text ] ] ]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0],
            Problem::Separator(protos::Separator::Colon)
        )
    );
    let fault = "Types\n[]\n[ Record!{ Text } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0],
            Problem::Separator(protos::Separator::Exclamation)
        )
    );
}

#[test]
fn rejects_a_bare_name_in_the_types_section_and_a_self_alias() {
    let fault = "Types\n[]\n[ Text Integer Bogus ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (vec![0, 0, 1, 0], Problem::Expected(Form::Declaration))
    );
    let fault = "Types\n[]\n[ Bogus.Bogus ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0],
            Problem::Cycle(protos::Text::try_from("Bogus").unwrap())
        )
    );
    let fault = "Types\n[]\n[ A.B B.Option<A> ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (
            vec![0, 0, 1, 0, 0],
            Problem::Cycle(protos::Text::try_from("A").unwrap())
        )
    );
}

#[test]
fn rejects_a_wrong_intrinsic_arity() {
    let fault = "Types\n[]\n[ Pair.Result<Text> ]\n[]".fault();
    assert_eq!(fault.problem(), (vec![0, 0, 1, 0, 0], Problem::Arity(2, 1)));
    let fault = "Types\n[]\n[ Own.{ Text<Integer> } ]\n[]".fault();
    assert_eq!(
        fault.problem(),
        (vec![0, 0, 1, 0, 0, 0], Problem::Arity(0, 1))
    );
}

#[test]
fn rejects_a_structural_fault_with_its_source_extent() {
    let source = "Types\n[]\n[ Record.{ Text ]\n[]";
    let fault = source.fault();
    let Situated(Situation { extent, .. }, ref inner) = fault;
    let Fault::Structural(structural) = inner else {
        panic!("expected a structural fault: {fault:?}");
    };
    assert_eq!(
        structural.problem,
        protos::Problem::Unopened(protos::Enclosure::Bracketed)
    );
    assert!(
        extent.0 < extent.1,
        "extent should be non-empty: {extent:?}"
    );
}

#[test]
fn deep_nesting_faults_typed_and_never_aborts() {
    let depth = 2000;
    let mut inner = "Text".to_owned();
    for _ in 0..depth {
        inner = format!("Vector<{inner}>");
    }
    let fault = format!("Types\n[]\n[ Deep.{inner} ]\n[]").fault();
    let (path, problem) = fault.problem();
    assert_eq!(problem, Problem::Depth);
    assert!(path.len() > 100);
    let mut inner = "X".to_owned();
    for _ in 0..depth {
        inner = format!("Wrap.[ {inner} ]");
    }
    let fault = format!("Types\n[]\n[ Deep.[ {inner} ] ]\n[]").fault();
    assert_eq!(fault.problem().1, Problem::Depth);
}

#[test]
fn moderate_nesting_reads_generates_and_round_trips() {
    let source = "Types\n[]\n[ Deep.Vector<Vector<Vector<Option<Result<Text Integer>>>>> Six.[ A.[ B.[ C.[ D.[ E.[ F.Integer ] ] ] ] ] ] ]\n[]";
    let rust = source.rust();
    assert!(rust.contains("pub enum SixABCDE"));
    let file = source.file();
    let again: File = Potential::<File>::from(file.textualize())
        .actualize()
        .unwrap();
    assert_eq!(file, again);
}

#[test]
fn a_lowercase_type_name_reads_as_written() {
    let File::Types(types) = "Types\n[]\n[ record.{ Text } ]\n[]".file() else {
        panic!("expected Types");
    };
    assert!(
        matches!(&types.types[0], TypeDeclaration::Struct(Identity { name, .. }, _) if name.0 == "record")
    );
}

#[test]
fn the_crate_reads_its_own_ethos() {
    let contract = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ethos-zero.ethos")).file();
    let File::Signal(signal) = &contract else {
        panic!("expected Signal");
    };
    assert_eq!(signal.requests.len(), 1);
    let faults = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fault.ethos")).file();
    assert!(matches!(faults, File::Types(_)));
}
