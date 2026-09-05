//! Every fixture's generated Rust, compiled against the pinned protos
//! and datomic and round-tripped through datom text. The modules under
//! tests/generated are committed; the freshness test regenerates them.

use datomic::{Datom, Datomic, Expected, Fault, Problem};
use protos::{Actualizable, Potential};

// ---------------------------------------------------------------------------
// The generated modules
// ---------------------------------------------------------------------------

#[rustfmt::skip]
#[path = "generated/capability-kinds.rs"]
mod capability_kinds;
#[rustfmt::skip]
#[path = "generated/entry-sema.rs"]
mod entry_sema;
#[rustfmt::skip]
#[path = "generated/multi-types.rs"]
mod multi_types;
#[rustfmt::skip]
#[path = "generated/orchestrate.rs"]
mod orchestrate;
#[rustfmt::skip]
#[path = "generated/placed-types.rs"]
mod placed_types;
#[rustfmt::skip]
#[path = "generated/processable-kinds.rs"]
mod processable_kinds;
#[rustfmt::skip]
#[path = "generated/record-types.rs"]
mod record_types;
#[rustfmt::skip]
#[path = "generated/sink-associations.rs"]
mod sink_associations;
#[rustfmt::skip]
#[path = "generated/streamable-kind.rs"]
mod streamable_kind;
#[rustfmt::skip]
#[path = "generated/tree-types.rs"]
mod tree_types;

// ---------------------------------------------------------------------------
// The enclosing module's companions: what the fixtures import from `super`
// ---------------------------------------------------------------------------

/// Streamable's superkind is the Fillable capability-kinds declares.
pub use capability_kinds::{Fillable, Summarizable};
/// Fillable's SinkError is the one sink-associations declares.
pub use sink_associations::SinkError;

/// The bound Streamable's Item carries.
pub trait Serializable {}

/// The interactions the sink-associations assertions demand, hand-written.
impl Summarizable for sink_associations::Sink {
    fn summarize(&self) -> protos::Text {
        self.0.clone()
    }
}

impl Fillable for sink_associations::Sink {
    fn push(&mut self, input: protos::Text) -> Result<protos::Integer, SinkError> {
        self.1.push(input);
        Ok(self.1.len() as protos::Integer)
    }

    fn drain(&mut self) -> Vec<protos::Text> {
        std::mem::take(&mut self.1)
    }

    fn create() -> Self {
        sink_associations::Sink(String::new(), Vec::new())
    }
}

/// A bearer of Streamable, to witness the trait compiles and is implementable.
struct Counter(Vec<protos::Text>);

impl Fillable for Counter {
    fn push(&mut self, input: protos::Text) -> Result<protos::Integer, SinkError> {
        self.0.push(input);
        Ok(0)
    }

    fn drain(&mut self) -> Vec<protos::Text> {
        std::mem::take(&mut self.0)
    }

    fn create() -> Self {
        Counter(Vec::new())
    }
}

impl Serializable for protos::Text {}

impl streamable_kind::Streamable for Counter {
    type Item = protos::Text;
    const CAPACITY: protos::Integer = 8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

/// A bearer of Processable, to witness the mapped bounds compile.
struct Processor;

impl processable_kinds::Processable<u8, u16> for Processor {
    fn process(&self) -> protos::Text {
        "processed".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Round trips: text -> value -> text, byte for byte
// ---------------------------------------------------------------------------

/// The kind whose capabilities read datom text as a type and write it back.
trait RoundTripping {
    fn read<T: Datomic>(&self) -> T
    where
        Datom: protos::Incorporable<T, Fault = Fault>;
    fn fault<T: Datomic>(&self) -> Fault
    where
        Datom: protos::Incorporable<T, Fault = Fault>;
    fn round_trips<T: Datomic>(&self)
    where
        Datom: protos::Incorporable<T, Fault = Fault>;
}

impl RoundTripping for str {
    fn read<T: Datomic>(&self) -> T
    where
        Datom: protos::Incorporable<T, Fault = Fault>,
    {
        match Potential::<T, Datom>::from(self).actualize() {
            Ok(value) => value,
            Err(fault) => panic!("{self} does not read: {fault:?}"),
        }
    }

    fn fault<T: Datomic>(&self) -> Fault
    where
        Datom: protos::Incorporable<T, Fault = Fault>,
    {
        match Potential::<T, Datom>::from(self).actualize() {
            Ok(_) => panic!("{self} was expected to fault"),
            Err(protos::Situated(_, fault)) => fault,
        }
    }

    fn round_trips<T: Datomic>(&self)
    where
        Datom: protos::Incorporable<T, Fault = Fault>,
    {
        let value: T = self.read();
        assert_eq!(value.textualize(), self);
    }
}

#[test]
fn orchestrate_requests_round_trip() {
    "Lock.{ MyLock 6329f1 [ /abs/path /abs/other ] \u{201C}why I hold it\u{201D} }"
        .round_trips::<orchestrate::Request>();
    "Release.42".round_trips::<orchestrate::Request>();
    "Observe.Locks".round_trips::<orchestrate::Request>();
}

#[test]
fn orchestrate_responses_round_trip() {
    "Locked.{ 442 MyLock 6329f1 [ /abs/path ] \u{201C}why I hold it\u{201D} }"
        .round_trips::<orchestrate::Response>();
    "Observed.Locks.[]".round_trips::<orchestrate::Response>();
    "Observed.Locks.[ { 7 Other f1 [ /abs/path ] r } ]".round_trips::<orchestrate::Response>();
    "LockRejected.PathOverlap.{ /abs/path { 7 Other f1 [ /abs/path ] r } }"
        .round_trips::<orchestrate::Response>();
    "ReleaseRejected.UnknownLockId".round_trips::<orchestrate::Response>();
}

#[test]
fn orchestrate_values_are_the_declared_shapes() {
    let request: orchestrate::Request = "Release.42".read();
    assert_eq!(request, orchestrate::Request::Release(42));
    let response: orchestrate::Response = "Observed.Locks.[]".read();
    assert_eq!(
        response,
        orchestrate::Response::Observed(orchestrate::Observation::Locks(vec![]))
    );
}

#[test]
fn multi_types_round_trip() {
    "{ Ada 1990 }".round_trips::<multi_types::Record>();
    "{ report [ 1 -2 3 ] }".round_trips::<multi_types::Report>();
    "Closed".round_trips::<multi_types::SinkError>();
    "Full".round_trips::<multi_types::SinkError>();
    "42".round_trips::<multi_types::LockId>();
    let record: record_types::Record = "{ Ada 1990 }".read();
    assert_eq!(record, record_types::Record("Ada".to_owned(), 1990));
}

#[test]
fn recursive_types_round_trip() {
    "Node.{ Leaf.1 Many.[ Leaf.2 Maybe.None ] }".round_trips::<tree_types::Tree>();
    "Maybe.Some.Leaf.3".round_trips::<tree_types::Tree>();
    "{ a Some.{ b None } }".round_trips::<tree_types::Chain>();
    "{ Tip Grow.{ Tip Tip } }".round_trips::<tree_types::Twin>();
    "[ Leaf.1 Node.{ Leaf.2 Leaf.3 } ]".round_trips::<tree_types::Forest>();
    let tree: tree_types::Tree = "Node.{ Leaf.1 Leaf.2 }".read();
    assert_eq!(
        tree,
        tree_types::Tree::Node(
            Box::new(tree_types::Tree::Leaf(1)),
            Box::new(tree_types::Tree::Leaf(2))
        )
    );
}

#[test]
fn containers_and_nesting_round_trip() {
    "{ Some.5 Ok.hello [ None Some.x ] }".round_trips::<tree_types::Wrapped>();
    "{ None Err.3 [] }".round_trips::<tree_types::Wrapped>();
    "A.X".round_trips::<tree_types::Nested>();
    "A.Y.7".round_trips::<tree_types::Nested>();
    "B.{ hi }".round_trips::<tree_types::Nested>();
    "[ [ [ Some.Ok.a None Some.Err.2 ] ] [] ]".round_trips::<tree_types::Deep>();
    let nested: tree_types::Nested = "A.Y.7".read();
    assert_eq!(nested, tree_types::Nested::A(tree_types::NestedA::Y(7)));
}

#[test]
fn constrained_type_round_trips_with_a_datomic_parameter() {
    "{ Some.1 2 }".round_trips::<placed_types::Placed<protos::Integer>>();
    "{ None [ a b ] }".round_trips::<placed_types::Placed<Vec<protos::Text>>>();
    "{ 3.5 True (a meaning \\(nested\\)) }".round_trips::<placed_types::Score>();
}

#[test]
fn sema_record_round_trips() {
    "{ db [ { k 1 } { j 2 } ] }".round_trips::<entry_sema::Record>();
    let record: entry_sema::Record = "{ db [] }".read();
    assert_eq!(record, entry_sema::Record("db".to_owned(), vec![]));
}

#[test]
fn sink_bears_its_kinds() {
    let mut sink = <sink_associations::Sink as Fillable>::create();
    assert_eq!(sink.push("a".to_owned()), Ok(1));
    assert_eq!(sink.drain(), vec!["a".to_owned()]);
    assert_eq!(sink.summarize(), "");
    let mut counter = <Counter as Fillable>::create();
    counter.push("x".to_owned()).unwrap();
    assert_eq!(
        streamable_kind::Streamable::next(&mut counter),
        Some("x".to_owned())
    );
    assert_eq!(<Counter as streamable_kind::Streamable>::CAPACITY, 8);
    assert_eq!(
        <Processor as processable_kinds::Processable<u8, u16>>::process(&Processor),
        "processed"
    );
}

// ---------------------------------------------------------------------------
// Faults: situated by path, the position index prepended at every level
// ---------------------------------------------------------------------------

#[test]
fn struct_position_fault_is_at_its_index() {
    let fault = "{ Ada notanumber }".fault::<multi_types::Record>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![1], Problem::Value("notanumber".to_owned()))
    );
}

#[test]
fn variant_body_position_fault_is_under_the_body() {
    let fault =
        "Locked.{ notanumber MyLock 6329f1 [ /abs/path ] r }".fault::<orchestrate::Response>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![0, 0], Problem::Value("notanumber".to_owned()))
    );
    let fault = "Node.{ Leaf.x Leaf.1 }".fault::<tree_types::Tree>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![0, 0, 0], Problem::Value("x".to_owned()))
    );
}

#[test]
fn vector_element_fault_carries_every_index() {
    let fault = "Observed.Locks.[ { 7 a b [] r } { x a b [] r } ]".fault::<orchestrate::Response>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![0, 0, 1, 0], Problem::Value("x".to_owned()))
    );
}

#[test]
fn arity_and_shape_faults_are_typed() {
    let fault = "Locked.{ 1 2 }".fault::<orchestrate::Response>();
    assert_eq!(fault, Fault::Corporate(vec![0], Problem::Arity(5, 2)));
    let fault = "Locked.[]".fault::<orchestrate::Response>();
    assert!(
        matches!(fault, Fault::Corporate(path, Problem::Shape(Expected::Struct, _)) if path == vec![0])
    );
    let fault = "Bogus.1".fault::<orchestrate::Response>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![], Problem::UnknownVariant("Bogus".to_owned()))
    );
    let fault = "Bogus".fault::<multi_types::SinkError>();
    assert_eq!(
        fault,
        Fault::Corporate(vec![], Problem::UnknownVariant("Bogus".to_owned()))
    );
    let fault = "{ 1 }".fault::<multi_types::SinkError>();
    assert!(
        matches!(fault, Fault::Corporate(path, Problem::Shape(Expected::Variant, _)) if path.is_empty())
    );
}
