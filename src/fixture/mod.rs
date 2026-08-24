//! The small Interface dialect used by the overnight realization fixture.
//!
//! This module gives each Interface section its own carrier and parsing
//! context.  The only structural machinery used here is the Protos walk.

use std::{collections::BTreeSet, path::PathBuf};

use protos::{
    Block, CursorObserving, Head, Headed, Realize, RealizeDriving, RealizeScope, RealizeScoping,
    RealizeWalk, Shape, ShapeDefined, SourceText, Textualize, TextualizeDriving, TextualizeScope,
    TextualizeScoping, TextualizeWalk, WalkFault, WalkObservation, WalkObserving,
};

use crate::build::{GeneratedArtifact, GeneratedArtifactOperations};

pub mod generated;

/// The exact fixture source owned by this crate.
pub const INTERFACE_SOURCE: &str = include_str!("../../fixtures/psyche/interface.ethos");

/// The version carried by an Interface header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// Text paired with the Interface dialect's realization context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceText {
    pub source: SourceText,
}

/// A complete five-section Interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interface {
    pub version: Version,
    pub channel: Option<Channel>,
    pub imports: Imports,
    pub inputs: Inputs,
    pub outputs: Outputs,
    pub refusals: Refusals,
    pub streams: Streams,
    pub types: Types,
}

/// The source-owned identity and fixed wire binding of an emitted Signal channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Channel {
    pub name: String,
    pub contract_id: u32,
    pub wire_revision: u16,
}

/// The import section and its placement-specific import entries.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Imports(pub Vec<Import>);

/// One named import path and its imported symbols.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub path: String,
    pub symbols: Vec<String>,
}

/// The Input section.  Its carrier is intentionally distinct from every
/// other section carrier.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Inputs(pub Vec<InputElement>);

/// One Input operation and its named payload type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputElement {
    pub operation: String,
    pub payload: String,
}

/// The Output section.  Its carrier is intentionally distinct from every
/// other section carrier.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Outputs(pub Vec<OutputElement>);

/// One Output operation and its named payload type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputElement {
    pub operation: String,
    pub payload: String,
}

/// The Refusal section.  Its carrier is intentionally distinct from every
/// other section carrier.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Refusals(pub Vec<RefusalElement>);

/// One Refusal operation and its named payload type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusalElement {
    pub operation: String,
    pub payload: String,
}

/// The Stream section.  Its carrier is intentionally distinct from every
/// other section carrier.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Streams(pub Vec<StreamElement>);

/// One Stream operation and its named payload type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamElement {
    pub operation: String,
    pub payload: String,
}

/// The Types section.  A declaration keeps its own body type all the way
/// through realization, even where a section uses the same structural shape.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Types(pub Vec<TypeElement>);

/// One named typedef, struct, or enum declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeElement {
    Typedef(NamedTypedef),
    Struct(NamedStruct),
    Enum(NamedEnum),
}

/// A named typedef declaration such as `Description.String`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedTypedef {
    pub name: String,
    pub target: String,
}

/// A named struct declaration such as `Entry.{Layer Description}`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedStruct {
    pub name: String,
    pub fields: Vec<String>,
}

/// A named enum declaration such as `Layer.[Spirit Intent Vision]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedEnum {
    pub name: String,
    pub variants: Vec<EnumVariantElement>,
}

/// One enum body's element.  This is not an Input, Output, Refusal, or Stream
/// element, even when its structural shape is also bare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumVariantElement {
    Data { variant: String, payload: String },
    Unit { variant: String },
}

/// A data-bearing scalar body carrier used only while applying a placement's
/// `ShapeDefined` selection before semantic symbol validation.
#[derive(Clone, Debug, Eq, PartialEq)]
struct BareValue(String);

/// Read-only transition evidence for one Interface realization or projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceEvidence {
    pub observation: WalkObservation,
    pub cursor: usize,
}

/// A realized Interface paired with the actual Protos walk that produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealizedInterface {
    pub value: Interface,
    pub evidence: InterfaceEvidence,
}

/// Canonical Interface text paired with the actual Protos walk that emitted it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedInterface {
    pub text: InterfaceText,
    pub evidence: InterfaceEvidence,
}

/// A dialect failure, including structural failures reported by Protos.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum InterfaceFault {
    #[error("protos walk: {0:?}")]
    Protos(WalkFault),
    #[error("unexpected Interface shape")]
    Shape,
    #[error("unexpected Interface head")]
    Head,
    #[error("unexpected Interface position")]
    Position,
    #[error("missing Interface position")]
    MissingPosition,
    #[error("extra Interface position")]
    ExtraPosition,
    #[error("invalid Interface version")]
    Version,
    #[error("invalid Interface channel binding")]
    Binding,
    #[error("invalid Interface symbol")]
    Symbol,
    #[error("invalid Interface member")]
    Member,
    #[error("duplicate Interface declaration")]
    Duplicate,
    #[error("unknown Interface type")]
    UnknownType,
}

impl From<WalkFault> for InterfaceFault {
    fn from(value: WalkFault) -> Self {
        Self::Protos(value)
    }
}

/// Access to a realized value and its walk evidence.
pub trait InterfaceRealizationViewing {
    fn value(&self) -> &Interface;
    fn evidence(&self) -> &InterfaceEvidence;
}

/// Access to canonical text and its walk evidence.
pub trait InterfaceProjectionViewing {
    fn text(&self) -> &InterfaceText;
    fn evidence(&self) -> &InterfaceEvidence;
}

/// The evidenced text-to-real direction for this dialect.
pub trait InterfaceEvidencedRealizing {
    fn realize_evidenced(&self) -> Result<RealizedInterface, InterfaceFault>;
}

/// The evidenced real-to-text direction for this dialect.
pub trait InterfaceEvidencedTextualizing {
    fn textualize_evidenced(&self) -> Result<ProjectedInterface, InterfaceFault>;
}

/// Projection of a checked Interface to its committed Rust fixture artifact.
pub trait RustArtifactProjecting {
    fn rust_source(&self) -> Result<String, InterfaceFault>;
    fn rust_artifact(&self, path: PathBuf) -> Result<GeneratedArtifact, InterfaceFault>;
}

/// Projection of a channel-bearing Interface to a complete Signal Rust module.
pub trait SignalArtifactProjecting {
    fn signal_rust_source(&self) -> Result<String, InterfaceFault>;
    fn signal_rust_artifact(&self, path: PathBuf) -> Result<GeneratedArtifact, InterfaceFault>;
}

impl InterfaceRealizationViewing for RealizedInterface {
    fn value(&self) -> &Interface {
        &self.value
    }

    fn evidence(&self) -> &InterfaceEvidence {
        &self.evidence
    }
}

impl InterfaceProjectionViewing for ProjectedInterface {
    fn text(&self) -> &InterfaceText {
        &self.text
    }

    fn evidence(&self) -> &InterfaceEvidence {
        &self.evidence
    }
}

impl ShapeDefined for Interface {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::DottedBraced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::DottedBraced && head == Some(&Head("Interface".into()))).then_some(())
    }
}

impl ShapeDefined for Channel {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::DottedBraced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::DottedBraced && head == Some(&Head("Channel".into()))).then_some(())
    }
}

impl ShapeDefined for Import {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::DottedSquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::DottedSquareBracketed && head.is_some()).then_some(())
    }
}

impl ShapeDefined for Imports {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for Inputs {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for Outputs {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for Refusals {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for Streams {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for Types {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::SquareBracketed]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::SquareBracketed && head.is_none()).then_some(())
    }
}

impl ShapeDefined for InputElement {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

impl ShapeDefined for OutputElement {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

impl ShapeDefined for RefusalElement {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

impl ShapeDefined for StreamElement {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeSelection {
    Typedef,
    Struct,
    Enum,
}

impl ShapeDefined for TypeElement {
    type Selection = TypeSelection;

    fn shapes() -> &'static [Shape] {
        &[
            Shape::Bare,
            Shape::DottedBraced,
            Shape::DottedSquareBracketed,
        ]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        match (shape, head) {
            (Shape::Bare, None) => Some(TypeSelection::Typedef),
            (Shape::DottedBraced, Some(_)) => Some(TypeSelection::Struct),
            (Shape::DottedSquareBracketed, Some(_)) => Some(TypeSelection::Enum),
            _ => None,
        }
    }
}

impl ShapeDefined for EnumVariantElement {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

impl ShapeDefined for BareValue {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Bare]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Bare && head.is_none()).then_some(())
    }
}

trait SymbolReading {
    fn symbol(value: &str) -> Result<String, InterfaceFault> {
        if value.is_empty()
            || !value
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_uppercase())
            || !value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(InterfaceFault::Symbol);
        }
        Ok(value.to_owned())
    }

    fn bare_value<T: ShapeDefined>(block: &Block) -> Result<BareValue, InterfaceFault> {
        T::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(BareValue(block.body.0.clone()))
    }

    fn bare_symbol<T: ShapeDefined>(block: &Block) -> Result<String, InterfaceFault> {
        let BareValue(value) = Self::bare_value::<T>(block)?;
        Self::symbol(&value)
    }

    fn head_symbol<T: ShapeDefined>(block: &Block) -> Result<(String, String), InterfaceFault> {
        T::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        let Some((head, symbol)) = block.body.0.split_once('.') else {
            return Err(InterfaceFault::Member);
        };
        if symbol.contains('.') {
            return Err(InterfaceFault::Member);
        }
        Ok((Self::symbol(head)?, Self::symbol(symbol)?))
    }

    fn type_reference(value: &str) -> Result<String, InterfaceFault> {
        if let Some(inner) = value
            .strip_prefix("Vector<")
            .and_then(|value| value.strip_suffix('>'))
        {
            if inner.is_empty() {
                return Err(InterfaceFault::Member);
            }
            return Ok(format!("Vector<{}>", Self::type_reference(inner)?));
        }
        Self::symbol(value)
    }
}

impl SymbolReading for String {}

trait VersionReading {
    fn read_header(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Version, InterfaceFault>;
    fn write_header(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

trait ChannelReading {
    fn read_channel(scope: &mut RealizeScope<'_>, block: &Block)
    -> Result<Channel, InterfaceFault>;
    fn write_channel(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl ChannelReading for Channel {
    fn read_channel(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Channel, InterfaceFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(InterfaceFault::Shape);
        }
        let values = scope.realize_body(&mut |_, child| String::bare_value::<BareValue>(child))?;
        if values.len() != 3 {
            return Err(if values.len() < 3 {
                InterfaceFault::MissingPosition
            } else {
                InterfaceFault::ExtraPosition
            });
        }
        let name = String::symbol(&values[0].0)?;
        let contract_id = values[1]
            .0
            .parse::<u32>()
            .map_err(|_| InterfaceFault::Binding)?;
        let wire_revision = values[2]
            .0
            .parse::<u16>()
            .map_err(|_| InterfaceFault::Binding)?;
        if contract_id == 0 || wire_revision == 0 {
            return Err(InterfaceFault::Binding);
        }
        Ok(Channel {
            name,
            contract_id,
            wire_revision,
        })
    }

    fn write_channel(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        if self.contract_id == 0 || self.wire_revision == 0 {
            return Err(InterfaceFault::Binding);
        }
        scope.textualize_block(Shape::DottedBraced, Some(&Head("Channel".into())), |body| {
            body.emit_scalar(&format!(
                "{} {} {}",
                self.name, self.contract_id, self.wire_revision
            ));
            Ok(())
        })
    }
}

impl VersionReading for Version {
    fn read_header(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Version, InterfaceFault> {
        if Interface::select(block.shape, block.head()).is_none() {
            return Err(InterfaceFault::Shape);
        }
        let mut values = Vec::new();
        scope.realize_body(&mut |_, child| {
            let BareValue(value) = String::bare_value::<BareValue>(child)?;
            let value = value.parse::<u16>().map_err(|_| InterfaceFault::Version)?;
            values.push(value);
            Ok::<(), InterfaceFault>(())
        })?;
        if values.len() != 3 {
            return Err(if values.len() < 3 {
                InterfaceFault::MissingPosition
            } else {
                InterfaceFault::ExtraPosition
            });
        }
        Ok(Version {
            major: values[0],
            minor: values[1],
            patch: values[2],
        })
    }

    fn write_header(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(
            Shape::DottedBraced,
            Some(&Head("Interface".into())),
            |body| {
                body.emit_scalar(&format!("{} {} {}", self.major, self.minor, self.patch));
                Ok(())
            },
        )
    }
}

trait ImportReading {
    fn realize_import(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Import, InterfaceFault>;
    fn textualize_import(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl ImportReading for Import {
    fn realize_import(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Import, InterfaceFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(InterfaceFault::Shape);
        }
        let path = block.head().ok_or(InterfaceFault::Head)?.0.clone();
        let symbols =
            scope.realize_body(&mut |_, child| String::bare_symbol::<BareValue>(child))?;
        Ok(Import { path, symbols })
    }

    fn textualize_import(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        let head = Head(self.path.clone());
        scope.textualize_block(Shape::DottedSquareBracketed, Some(&head), |body| {
            for symbol in &self.symbols {
                body.textualize_block::<(), InterfaceFault, _>(Shape::Bare, None, |child| {
                    child.emit_scalar(symbol);
                    Ok(())
                })?;
            }
            Ok(())
        })
    }
}

trait SectionReading<T> {
    fn realize_section(scope: &mut RealizeScope<'_>, block: &Block) -> Result<T, InterfaceFault>;
}

trait SectionWriting {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl SectionReading<Inputs> for Inputs {
    fn realize_section(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Inputs, InterfaceFault> {
        Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(Inputs(scope.realize_body(&mut |child_scope, child| {
            <InputElement as InputContext>::realize_input(child_scope, child)
        })?))
    }
}

impl SectionWriting for Inputs {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for element in &self.0 {
                <InputElement as InputContext>::textualize_input(element, body)?;
            }
            Ok(())
        })
    }
}

impl SectionReading<Outputs> for Outputs {
    fn realize_section(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Outputs, InterfaceFault> {
        Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(Outputs(scope.realize_body(&mut |child_scope, child| {
            <OutputElement as OutputContext>::realize_output(child_scope, child)
        })?))
    }
}

impl SectionWriting for Outputs {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for element in &self.0 {
                <OutputElement as OutputContext>::textualize_output(element, body)?;
            }
            Ok(())
        })
    }
}

impl SectionReading<Refusals> for Refusals {
    fn realize_section(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Refusals, InterfaceFault> {
        Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(Refusals(scope.realize_body(
            &mut |child_scope, child| {
                <RefusalElement as RefusalContext>::realize_refusal(child_scope, child)
            },
        )?))
    }
}

impl SectionWriting for Refusals {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for element in &self.0 {
                <RefusalElement as RefusalContext>::textualize_refusal(element, body)?;
            }
            Ok(())
        })
    }
}

impl SectionReading<Streams> for Streams {
    fn realize_section(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Streams, InterfaceFault> {
        Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(Streams(scope.realize_body(&mut |child_scope, child| {
            <StreamElement as StreamContext>::realize_stream(child_scope, child)
        })?))
    }
}

impl SectionWriting for Streams {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for element in &self.0 {
                <StreamElement as StreamContext>::textualize_stream(element, body)?;
            }
            Ok(())
        })
    }
}

impl SectionReading<Types> for Types {
    fn realize_section(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Types, InterfaceFault> {
        Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        Ok(Types(scope.realize_body(&mut |child_scope, child| {
            <TypeElement as TypeElementReading>::realize_type(child_scope, child)
        })?))
    }
}

impl SectionWriting for Types {
    fn textualize_section(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::SquareBracketed, None, |body| {
            for element in &self.0 {
                element.textualize_type(body)?;
            }
            Ok(())
        })
    }
}

trait OperationElementReading: Sized {
    fn realize_operation(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Self, InterfaceFault>;
    fn textualize_operation(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl OperationElementReading for InputElement {
    fn realize_operation(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        let (operation, payload) = String::head_symbol::<InputElement>(block)?;
        Ok(Self { operation, payload })
    }

    fn textualize_operation(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&format!("{}.{}", self.operation, self.payload));
            Ok(())
        })
    }
}

trait InputContext {
    fn realize_input(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<InputElement, InterfaceFault>;
    fn textualize_input(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl InputContext for InputElement {
    fn realize_input(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        <Self as OperationElementReading>::realize_operation(scope, block)
    }

    fn textualize_input(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        <Self as OperationElementReading>::textualize_operation(self, scope)
    }
}

impl OperationElementReading for OutputElement {
    fn realize_operation(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        let (operation, payload) = String::head_symbol::<OutputElement>(block)?;
        Ok(Self { operation, payload })
    }

    fn textualize_operation(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&format!("{}.{}", self.operation, self.payload));
            Ok(())
        })
    }
}

trait OutputContext {
    fn realize_output(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<OutputElement, InterfaceFault>;
    fn textualize_output(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl OutputContext for OutputElement {
    fn realize_output(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        <Self as OperationElementReading>::realize_operation(scope, block)
    }

    fn textualize_output(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        <Self as OperationElementReading>::textualize_operation(self, scope)
    }
}

impl OperationElementReading for RefusalElement {
    fn realize_operation(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        let (operation, payload) = String::head_symbol::<RefusalElement>(block)?;
        Ok(Self { operation, payload })
    }

    fn textualize_operation(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&format!("{}.{}", self.operation, self.payload));
            Ok(())
        })
    }
}

trait RefusalContext {
    fn realize_refusal(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<RefusalElement, InterfaceFault>;
    fn textualize_refusal(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl RefusalContext for RefusalElement {
    fn realize_refusal(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<Self, InterfaceFault> {
        <Self as OperationElementReading>::realize_operation(scope, block)
    }

    fn textualize_refusal(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        <Self as OperationElementReading>::textualize_operation(self, scope)
    }
}

impl OperationElementReading for StreamElement {
    fn realize_operation(_: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        let (operation, payload) = String::head_symbol::<StreamElement>(block)?;
        Ok(Self { operation, payload })
    }

    fn textualize_operation(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&format!("{}.{}", self.operation, self.payload));
            Ok(())
        })
    }
}

trait StreamContext {
    fn realize_stream(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<StreamElement, InterfaceFault>;
    fn textualize_stream(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl StreamContext for StreamElement {
    fn realize_stream(scope: &mut RealizeScope<'_>, block: &Block) -> Result<Self, InterfaceFault> {
        <Self as OperationElementReading>::realize_operation(scope, block)
    }

    fn textualize_stream(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        <Self as OperationElementReading>::textualize_operation(self, scope)
    }
}

trait TypeElementReading {
    fn realize_type(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<TypeElement, InterfaceFault>;
    fn textualize_type(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl TypeElementReading for TypeElement {
    fn realize_type(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<TypeElement, InterfaceFault> {
        match Self::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)? {
            TypeSelection::Typedef => {
                let (name, target) = block.body.0.split_once('.').ok_or(InterfaceFault::Member)?;
                if target.contains('.') {
                    return Err(InterfaceFault::Member);
                }
                let name = String::symbol(name)?;
                let target = String::type_reference(target)?;
                Ok(TypeElement::Typedef(NamedTypedef { name, target }))
            }
            TypeSelection::Struct => {
                let name = String::symbol(&block.head().ok_or(InterfaceFault::Head)?.0)?;
                let fields =
                    scope.realize_body(&mut |_, child| String::bare_symbol::<BareValue>(child))?;
                Ok(TypeElement::Struct(NamedStruct { name, fields }))
            }
            TypeSelection::Enum => {
                let name = String::symbol(&block.head().ok_or(InterfaceFault::Head)?.0)?;
                let variants = scope.realize_body(&mut |child_scope, child| {
                    <EnumVariantElement as EnumVariantReading>::realize_variant(child_scope, child)
                })?;
                Ok(TypeElement::Enum(NamedEnum { name, variants }))
            }
        }
    }

    fn textualize_type(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        match self {
            TypeElement::Typedef(value) => scope.textualize_block(Shape::Bare, None, |body| {
                body.emit_scalar(&format!("{}.{}", value.name, value.target));
                Ok(())
            }),
            TypeElement::Struct(value) => {
                let head = Head(value.name.clone());
                scope.textualize_block(Shape::DottedBraced, Some(&head), |body| {
                    for field in &value.fields {
                        body.textualize_block::<(), InterfaceFault, _>(
                            Shape::Bare,
                            None,
                            |child| {
                                child.emit_scalar(field);
                                Ok(())
                            },
                        )?;
                    }
                    Ok(())
                })
            }
            TypeElement::Enum(value) => {
                let head = Head(value.name.clone());
                scope.textualize_block(Shape::DottedSquareBracketed, Some(&head), |body| {
                    for variant in &value.variants {
                        variant.textualize_variant(body)?;
                    }
                    Ok(())
                })
            }
        }
    }
}

trait EnumVariantReading {
    fn realize_variant(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<EnumVariantElement, InterfaceFault>;
    fn textualize_variant(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl EnumVariantReading for EnumVariantElement {
    fn realize_variant(
        _: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<EnumVariantElement, InterfaceFault> {
        if Self::select(block.shape, block.head()).is_none() {
            return Err(InterfaceFault::Shape);
        }
        if block.body.0.contains('.') {
            let (variant, payload) = String::head_symbol::<EnumVariantElement>(block)?;
            Ok(EnumVariantElement::Data { variant, payload })
        } else {
            Ok(EnumVariantElement::Unit {
                variant: String::bare_symbol::<EnumVariantElement>(block)?,
            })
        }
    }

    fn textualize_variant(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        let value = match self {
            EnumVariantElement::Data { variant, payload } => format!("{}.{}", variant, payload),
            EnumVariantElement::Unit { variant } => variant.clone(),
        };
        scope.textualize_block(Shape::Bare, None, |body| {
            body.emit_scalar(&value);
            Ok(())
        })
    }
}

#[derive(Default)]
struct InterfaceState {
    version: Option<Version>,
    channel: Option<Channel>,
    imports: Option<Imports>,
    sections: Option<InterfaceSections>,
}

struct InterfaceSections {
    inputs: Inputs,
    outputs: Outputs,
    refusals: Refusals,
    streams: Streams,
    types: Types,
}

impl ShapeDefined for InterfaceSections {
    type Selection = ();

    fn shapes() -> &'static [Shape] {
        &[Shape::Braced]
    }

    fn select(shape: Shape, head: Option<&Head>) -> Option<Self::Selection> {
        (shape == Shape::Braced && head.is_none()).then_some(())
    }
}

trait InterfaceDocumentReading {
    fn realize_root_block(
        &mut self,
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<(), InterfaceFault>;
    fn finish(self) -> Result<Interface, InterfaceFault>;
}

impl InterfaceDocumentReading for InterfaceState {
    fn realize_root_block(
        &mut self,
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<(), InterfaceFault> {
        if self.version.is_none() {
            self.version = Some(Version::read_header(scope, block)?);
        } else if self.imports.is_none()
            && self.channel.is_none()
            && Channel::select(block.shape, block.head()).is_some()
        {
            self.channel = Some(Channel::read_channel(scope, block)?);
        } else if self.imports.is_none() {
            Imports::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
            self.imports = Some(Imports(scope.realize_body(&mut |child_scope, child| {
                Import::realize_import(child_scope, child)
            })?));
        } else if self.sections.is_none() {
            InterfaceSections::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
            self.sections = Some(InterfaceSections::realize_sections(scope, block)?);
        } else {
            return Err(InterfaceFault::ExtraPosition);
        }
        Ok(())
    }

    fn finish(self) -> Result<Interface, InterfaceFault> {
        let sections = self.sections.ok_or(InterfaceFault::MissingPosition)?;
        Ok(Interface {
            version: self.version.ok_or(InterfaceFault::MissingPosition)?,
            channel: self.channel,
            imports: self.imports.ok_or(InterfaceFault::MissingPosition)?,
            inputs: sections.inputs,
            outputs: sections.outputs,
            refusals: sections.refusals,
            streams: sections.streams,
            types: sections.types,
        })
    }
}

trait SectionsReading {
    fn realize_sections(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<InterfaceSections, InterfaceFault>;
}

impl SectionsReading for InterfaceSections {
    fn realize_sections(
        scope: &mut RealizeScope<'_>,
        block: &Block,
    ) -> Result<InterfaceSections, InterfaceFault> {
        InterfaceSections::select(block.shape, block.head()).ok_or(InterfaceFault::Shape)?;
        let mut position = 0;
        let mut inputs = None;
        let mut outputs = None;
        let mut refusals = None;
        let mut streams = None;
        let mut types = None;
        scope.realize_body(&mut |child_scope, child| {
            let current = position;
            position += 1;
            match current {
                0 => inputs = Some(Inputs::realize_section(child_scope, child)?),
                1 => outputs = Some(Outputs::realize_section(child_scope, child)?),
                2 => refusals = Some(Refusals::realize_section(child_scope, child)?),
                3 => streams = Some(Streams::realize_section(child_scope, child)?),
                4 => types = Some(Types::realize_section(child_scope, child)?),
                _ => return Err(InterfaceFault::ExtraPosition),
            }
            Ok(())
        })?;
        if position != 5 {
            return Err(InterfaceFault::MissingPosition);
        }
        Ok(InterfaceSections {
            inputs: inputs.ok_or(InterfaceFault::MissingPosition)?,
            outputs: outputs.ok_or(InterfaceFault::MissingPosition)?,
            refusals: refusals.ok_or(InterfaceFault::MissingPosition)?,
            streams: streams.ok_or(InterfaceFault::MissingPosition)?,
            types: types.ok_or(InterfaceFault::MissingPosition)?,
        })
    }
}

trait InterfaceDocumentWriting {
    fn textualize_document(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault>;
}

impl InterfaceDocumentWriting for Interface {
    fn textualize_document(&self, scope: &mut TextualizeScope<'_>) -> Result<(), InterfaceFault> {
        self.version.write_header(scope)?;
        if let Some(channel) = &self.channel {
            channel.write_channel(scope)?;
        }
        scope.textualize_block::<(), InterfaceFault, _>(Shape::SquareBracketed, None, |body| {
            for import in &self.imports.0 {
                import.textualize_import(body)?;
            }
            Ok(())
        })?;
        scope.textualize_block(Shape::Braced, None, |body| {
            self.inputs.textualize_section(body)?;
            self.outputs.textualize_section(body)?;
            self.refusals.textualize_section(body)?;
            self.streams.textualize_section(body)?;
            self.types.textualize_section(body)
        })
    }
}

trait InterfaceValidation {
    fn validate(&self) -> Result<(), InterfaceFault>;

    fn knows_type(names: &BTreeSet<String>, value: &str) -> bool;
}

impl InterfaceValidation for Interface {
    fn knows_type(names: &BTreeSet<String>, value: &str) -> bool {
        value == "String"
            || value == "Integer"
            || names.contains(value)
            || value
                .strip_prefix("Vector<")
                .and_then(|inner| inner.strip_suffix('>'))
                .is_some_and(|inner| !inner.is_empty() && Self::knows_type(names, inner))
    }

    fn validate(&self) -> Result<(), InterfaceFault> {
        let mut names = BTreeSet::new();
        for declaration in &self.types.0 {
            let name = match declaration {
                TypeElement::Typedef(value) => &value.name,
                TypeElement::Struct(value) => &value.name,
                TypeElement::Enum(value) => &value.name,
            };
            String::symbol(name)?;
            if !names.insert(name.clone()) {
                return Err(InterfaceFault::Duplicate);
            }
        }
        let known = |value: &str| Self::knows_type(&names, value);
        let mut input_names = BTreeSet::new();
        for operation in &self.inputs.0 {
            String::symbol(&operation.operation)?;
            if !input_names.insert(operation.operation.clone()) {
                return Err(InterfaceFault::Duplicate);
            }
            if !names.contains(&operation.payload) {
                return Err(InterfaceFault::UnknownType);
            }
        }
        let mut output_names = BTreeSet::new();
        for operation in &self.outputs.0 {
            String::symbol(&operation.operation)?;
            if !output_names.insert(operation.operation.clone()) {
                return Err(InterfaceFault::Duplicate);
            }
            if !names.contains(&operation.payload) {
                return Err(InterfaceFault::UnknownType);
            }
        }
        let mut refusal_names = BTreeSet::new();
        for operation in &self.refusals.0 {
            String::symbol(&operation.operation)?;
            if !refusal_names.insert(operation.operation.clone()) {
                return Err(InterfaceFault::Duplicate);
            }
            if !names.contains(&operation.payload) {
                return Err(InterfaceFault::UnknownType);
            }
        }
        let mut stream_names = BTreeSet::new();
        for operation in &self.streams.0 {
            String::symbol(&operation.operation)?;
            if !stream_names.insert(operation.operation.clone()) {
                return Err(InterfaceFault::Duplicate);
            }
            if !names.contains(&operation.payload) {
                return Err(InterfaceFault::UnknownType);
            }
        }
        for declaration in &self.types.0 {
            match declaration {
                TypeElement::Typedef(value) if !known(&value.target) => {
                    return Err(InterfaceFault::UnknownType);
                }
                TypeElement::Typedef(value) => {
                    String::type_reference(&value.target)?;
                }
                TypeElement::Struct(value) => {
                    let mut fields = BTreeSet::new();
                    if value.fields.iter().any(|field| !known(field)) {
                        return Err(InterfaceFault::UnknownType);
                    }
                    for field in &value.fields {
                        String::symbol(field)?;
                        if !fields.insert(field.clone()) {
                            return Err(InterfaceFault::Duplicate);
                        }
                    }
                }
                TypeElement::Enum(value) => {
                    if value.variants.iter().any(|variant| {
                        matches!(variant, EnumVariantElement::Data { payload, .. } if !known(payload))
                    }) {
                        return Err(InterfaceFault::UnknownType);
                    }
                    let mut variants = BTreeSet::new();
                    for variant in &value.variants {
                        let name = match variant {
                            EnumVariantElement::Data { variant, .. }
                            | EnumVariantElement::Unit { variant } => variant,
                        };
                        String::symbol(name)?;
                        if !variants.insert(name.clone()) {
                            return Err(InterfaceFault::Duplicate);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl InterfaceEvidencedRealizing for InterfaceText {
    fn realize_evidenced(&self) -> Result<RealizedInterface, InterfaceFault> {
        let mut walk = RealizeWalk::default();
        let mut state = InterfaceState::default();
        walk.realize_source(&self.source, |scope, block| {
            state.realize_root_block(scope, block)
        })?;
        let value = state.finish()?;
        value.validate()?;
        Ok(RealizedInterface {
            value,
            evidence: InterfaceEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl Realize for InterfaceText {
    type Real = Interface;
    type Fault = InterfaceFault;

    fn realize(&self) -> Result<Self::Real, Self::Fault> {
        self.realize_evidenced().map(|value| value.value)
    }
}

impl InterfaceEvidencedTextualizing for Interface {
    fn textualize_evidenced(&self) -> Result<ProjectedInterface, InterfaceFault> {
        self.validate()?;
        let mut walk = TextualizeWalk::default();
        walk.textualize_source(|scope| self.textualize_document(scope))?;
        Ok(ProjectedInterface {
            text: InterfaceText {
                source: walk.textual_source(),
            },
            evidence: InterfaceEvidence {
                observation: walk.observation(),
                cursor: walk.cursor(),
            },
        })
    }
}

impl Textualize for Interface {
    type Textual = Result<InterfaceText, InterfaceFault>;

    fn textualize(&self) -> Self::Textual {
        self.textualize_evidenced().map(|value| value.text)
    }
}

trait RustTypeWriting {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault>;

    fn write_rust_structural(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("#[derive(Archi");
        output.push_str("ve, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]\n");
        self.write_rust(output)
    }

    fn write_rust_derived(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("#[derive(Archi");
        output.push_str("ve, RkyvSerialize, RkyvDeserialize, ::dotos::DotosEn");
        output.push_str("co");
        output.push_str("de, ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("de, Debug, Clone, PartialEq, Eq)]\n");
        self.write_rust(output)
    }
}

trait NamedDotosTextWriting {
    fn write_named_dotos_text(&self, output: &mut String) -> Result<(), InterfaceFault>;
}

trait RustNameWriting {
    fn rust_type_name(&self) -> String;
    fn rust_field_name(&self) -> String;
}

impl RustNameWriting for str {
    fn rust_type_name(&self) -> String {
        match self {
            "String" => "String".into(),
            "Integer" => "i64".into(),
            value if value.starts_with("Vector<") && value.ends_with('>') => {
                let inner = &value["Vector<".len()..value.len() - 1];
                format!("Vec<{}>", inner.rust_type_name())
            }
            value => value.into(),
        }
    }

    fn rust_field_name(&self) -> String {
        let characters = self.chars().collect::<Vec<_>>();
        let mut output = String::new();
        for (index, character) in characters.iter().copied().enumerate() {
            if character == '_' {
                if !output.ends_with('_') {
                    output.push('_');
                }
                continue;
            }
            if character.is_ascii_uppercase() {
                let previous_is_lowercase = index > 0 && characters[index - 1].is_ascii_lowercase();
                let next_is_lowercase = characters
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_lowercase());
                if !output.is_empty()
                    && !output.ends_with('_')
                    && (previous_is_lowercase || next_is_lowercase)
                {
                    output.push('_');
                }
                output.push(character.to_ascii_lowercase());
            } else {
                output.push(character);
            }
        }
        output
    }
}

impl RustTypeWriting for InputElement {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("    ");
        output.push_str(&self.operation);
        output.push('(');
        output.push_str(&self.payload);
        output.push_str("),\n");
        Ok(())
    }
}

impl RustTypeWriting for OutputElement {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("    ");
        output.push_str(&self.operation);
        output.push('(');
        output.push_str(&self.payload);
        output.push_str("),\n");
        Ok(())
    }
}

impl RustTypeWriting for RefusalElement {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("    ");
        output.push_str(&self.operation);
        output.push('(');
        output.push_str(&self.payload);
        output.push_str("),\n");
        Ok(())
    }
}

impl RustTypeWriting for StreamElement {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("    ");
        output.push_str(&self.operation);
        output.push('(');
        output.push_str(&self.payload);
        output.push_str("),\n");
        Ok(())
    }
}

impl RustTypeWriting for NamedTypedef {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("pub struct ");
        output.push_str(&self.name);
        output.push_str("(pub ");
        output.push_str(&self.target.as_str().rust_type_name());
        output.push_str(");\n\n");
        Ok(())
    }

    fn write_rust_derived(&self, output: &mut String) -> Result<(), InterfaceFault> {
        self.write_rust_structural(output)?;
        self.write_named_dotos_text(output)
    }
}

impl RustTypeWriting for NamedStruct {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("pub struct ");
        output.push_str(&self.name);
        output.push_str(" {\n");
        for field in &self.fields {
            output.push_str("    pub ");
            output.push_str(&field.as_str().rust_field_name());
            output.push_str(": ");
            output.push_str(field);
            output.push_str(",\n");
        }
        output.push_str("}\n\n");
        Ok(())
    }

    fn write_rust_derived(&self, output: &mut String) -> Result<(), InterfaceFault> {
        self.write_rust_structural(output)?;
        self.write_named_dotos_text(output)
    }
}

impl NamedDotosTextWriting for NamedTypedef {
    fn write_named_dotos_text(&self, output: &mut String) -> Result<(), InterfaceFault> {
        let target = self.target.as_str().rust_type_name();
        output.push_str("impl ::dotos::DotosEn");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn to_dotos(&self) -> String {\n        format!(\"");
        output.push_str(&self.name);
        output.push_str(".{}\", <");
        output.push_str(&target);
        output.push_str(" as ::dotos::DotosEn");
        output.push_str("co");
        output.push_str("de>::to_dotos(&self.0))\n    }\n}\n\n");
        output.push_str("impl ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(
            " {\n    fn from_dotos_block(block: &::dotos::Block) -> Result<Self, ::dotos::DotosDe",
        );
        output.push_str("co");
        output.push_str("deError> {\n        let (head, payload) = block.as_application().ok_or(::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError::ExpectedDelimited {\n            type_name: \"");
        output.push_str(&self.name);
        output.push_str("\",\n            delimiter: \"named dotted application\",\n        })?;\n        let head = head.demote_to_string().ok_or(::dotos::DotosDe");
        output.push_str("co");
        output.push_str(
            "deError::ExpectedAtom { type_name: \"named type head\" })?;\n        if head != \"",
        );
        output.push_str(&self.name);
        output.push_str("\" {\n            return Err(::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError::UnknownVariant { enum_name: \"");
        output.push_str(&self.name);
        output.push_str("\", variant: head.to_owned() });\n        }\n        Ok(Self(<");
        output.push_str(&target);
        output.push_str(" as ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("de>::from_dotos_block(payload)?))\n    }\n}\n\n");
        output.push_str("impl ::dotos::DotosBodyEn");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn to_dotos_body(&self) -> ::dotos::DotosBodyEncoding {\n        ::dotos::DotosBodyEncoding::new(vec![::dotos::DotosEn");
        output.push_str("co");
        output.push_str("de::to_dotos(self)])\n    }\n}\n\n");
        output.push_str("impl ::dotos::DotosBodyDe");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn from_dotos_body(body: &::dotos::DotosBody<'_>) -> Result<Self, ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError> {\n        let fields = body.expect_fields(\"");
        output.push_str(&self.name);
        output.push_str("\", 1)?;\n        <Self as ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("de>::from_dotos_block(&fields[0])\n    }\n}\n\n");
        Ok(())
    }
}

impl NamedDotosTextWriting for NamedStruct {
    fn write_named_dotos_text(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("impl ::dotos::DotosBodyEn");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn to_dotos_body(&self) -> ::dotos::DotosBodyEncoding {\n        ::dotos::DotosBodyEncoding::new(vec![\n");
        for field in &self.fields {
            output.push_str("            ::dotos::DotosEn");
            output.push_str("co");
            output.push_str("de::to_dotos(&self.");
            output.push_str(&field.as_str().rust_field_name());
            output.push_str("),\n");
        }
        output.push_str("        ])\n    }\n}\n\n");
        output.push_str("impl ::dotos::DotosEn");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn to_dotos(&self) -> String {\n        format!(\"");
        output.push_str(&self.name);
        output.push_str(".{}\", <Self as ::dotos::DotosBodyEn");
        output.push_str("co");
        output.push_str(
            "de>::to_dotos_body(self).to_delimited_dotos(::dotos::Delimiter::Brace))\n    }\n}\n\n",
        );
        output.push_str("impl ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(
            " {\n    fn from_dotos_block(block: &::dotos::Block) -> Result<Self, ::dotos::DotosDe",
        );
        output.push_str("co");
        output.push_str("deError> {\n        let (head, payload) = block.as_application().ok_or(::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError::ExpectedDelimited {\n            type_name: \"");
        output.push_str(&self.name);
        output.push_str("\",\n            delimiter: \"named dotted application\",\n        })?;\n        let head = head.demote_to_string().ok_or(::dotos::DotosDe");
        output.push_str("co");
        output.push_str(
            "deError::ExpectedAtom { type_name: \"named type head\" })?;\n        if head != \"",
        );
        output.push_str(&self.name);
        output.push_str("\" {\n            return Err(::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError::UnknownVariant { enum_name: \"");
        output.push_str(&self.name);
        output.push_str("\", variant: head.to_owned() });\n        }\n        let body = ::dotos::DotosBlock::new(payload).expect_body(::dotos::Delimiter::Brace, \"");
        output.push_str(&self.name);
        output.push_str("\")?;\n        <Self as ::dotos::DotosBodyDe");
        output.push_str("co");
        output.push_str("de>::from_dotos_body(&body)\n    }\n}\n\n");
        output.push_str("impl ::dotos::DotosBodyDe");
        output.push_str("co");
        output.push_str("de for ");
        output.push_str(&self.name);
        output.push_str(" {\n    fn from_dotos_body(body: &::dotos::DotosBody<'_>) -> Result<Self, ::dotos::DotosDe");
        output.push_str("co");
        output.push_str("deError> {\n        let fields = body.expect_fields(\"");
        output.push_str(&self.name);
        output.push_str("\", ");
        output.push_str(&self.fields.len().to_string());
        output.push_str(")?;\n        Ok(Self {\n");
        for (index, field) in self.fields.iter().enumerate() {
            output.push_str("            ");
            output.push_str(&field.as_str().rust_field_name());
            output.push_str(": <");
            output.push_str(&field.as_str().rust_type_name());
            output.push_str(" as ::dotos::DotosDe");
            output.push_str("co");
            output.push_str("de>::from_dotos_block(&fields[");
            output.push_str(&index.to_string());
            output.push_str("])?,\n");
        }
        output.push_str("        })\n    }\n}\n\n");
        Ok(())
    }
}

impl RustTypeWriting for NamedEnum {
    fn write_rust(&self, output: &mut String) -> Result<(), InterfaceFault> {
        output.push_str("pub enum ");
        output.push_str(&self.name);
        output.push_str(" {\n");
        for variant in &self.variants {
            output.push_str("    ");
            match variant {
                EnumVariantElement::Data { variant, payload } => {
                    output.push_str(variant);
                    output.push('(');
                    output.push_str(payload);
                    output.push_str("),\n");
                }
                EnumVariantElement::Unit { variant } => {
                    output.push_str(variant);
                    output.push_str(",\n");
                }
            }
        }
        output.push_str("}\n\n");
        Ok(())
    }
}

impl RustArtifactProjecting for Interface {
    fn rust_source(&self) -> Result<String, InterfaceFault> {
        self.validate()?;
        let mut output = String::from("// @generated by ethos-monolith from Ethos source\n\n");
        output.push_str("pub enum Input {\n");
        for value in &self.inputs.0 {
            value.write_rust(&mut output)?;
        }
        output.push_str("}\n\n");
        output.push_str("pub enum Output {\n");
        for value in &self.outputs.0 {
            value.write_rust(&mut output)?;
        }
        output.push_str("}\n\n");
        output.push_str("pub enum Refusal {\n");
        for value in &self.refusals.0 {
            value.write_rust(&mut output)?;
        }
        output.push_str("}\n\n");
        output.push_str("pub enum Stream {\n");
        for value in &self.streams.0 {
            value.write_rust(&mut output)?;
        }
        output.push_str("}\n\n");
        for declaration in &self.types.0 {
            match declaration {
                TypeElement::Typedef(value) => value.write_rust(&mut output)?,
                TypeElement::Struct(value) => value.write_rust(&mut output)?,
                TypeElement::Enum(value) => value.write_rust(&mut output)?,
            }
        }
        if output.ends_with("\n\n") {
            output.pop();
        }
        Ok(output)
    }

    fn rust_artifact(&self, path: PathBuf) -> Result<GeneratedArtifact, InterfaceFault> {
        Ok(GeneratedArtifact::new(path, self.rust_source()?))
    }
}

impl SignalArtifactProjecting for Interface {
    fn signal_rust_source(&self) -> Result<String, InterfaceFault> {
        self.validate()?;
        if !self.imports.0.is_empty() {
            return Err(InterfaceFault::Shape);
        }
        let channel = self.channel.as_ref().ok_or(InterfaceFault::Binding)?;
        let wire_name = format!("{}Wire", channel.name);
        let reply_name = format!("{}Reply", channel.name);
        let request_name = format!("{}Request", channel.name);
        let mut output = String::from("// @generated by ethos-monolith from Ethos source\n\n");
        output.push_str("use core::num::{NonZeroU16, NonZeroU32};\n");
        output.push_str("use rkyv::{Archi");
        output.push_str("ve, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};\n");
        output.push_str("use signal_frame::{signal_channel, ContractBinding, ContractId, WireContract, WireRevision};\n\n");
        output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ");
        output.push_str(&wire_name);
        output.push_str(" {}\n\nimpl WireContract for ");
        output.push_str(&wire_name);
        output.push_str(" {\n    const BINDING: ContractBinding = ContractBinding::new(\n        ContractId::new(NonZeroU32::new(");
        output.push_str(&channel.contract_id.to_string());
        output.push_str(").expect(\"Ethos Channel contract id is nonzero\")),\n        WireRevision::new(NonZeroU16::new(");
        output.push_str(&channel.wire_revision.to_string());
        output.push_str(").expect(\"Ethos Channel wire revision is nonzero\")),\n    );\n}\n\n");
        for declaration in &self.types.0 {
            match declaration {
                TypeElement::Typedef(value) => value.write_rust_derived(&mut output)?,
                TypeElement::Struct(value) => value.write_rust_derived(&mut output)?,
                TypeElement::Enum(value) => value.write_rust_derived(&mut output)?,
            }
        }
        if !self.refusals.0.is_empty() {
            output.push_str("#[derive(Archi");
            output.push_str("ve, RkyvSerialize, RkyvDeserialize, ::dotos::DotosEn");
            output.push_str("co");
            output.push_str("de, ::dotos::DotosDe");
            output.push_str("co");
            output.push_str("de, Debug, Clone, PartialEq, Eq)]\npub enum Refusal {\n");
            for value in &self.refusals.0 {
                value.write_rust(&mut output)?;
            }
            output.push_str("}\n\n");
        }
        if !self.streams.0.is_empty() {
            output.push_str("#[derive(Archi");
            output.push_str("ve, RkyvSerialize, RkyvDeserialize, ::dotos::DotosEn");
            output.push_str("co");
            output.push_str("de, ::dotos::DotosDe");
            output.push_str("co");
            output.push_str("de, Debug, Clone, PartialEq, Eq)]\npub enum Stream {\n");
            for value in &self.streams.0 {
                value.write_rust(&mut output)?;
            }
            output.push_str("}\n\n");
        }
        output.push_str("signal_channel! {\n    channel ");
        output.push_str(&channel.name);
        output.push_str(" contract ");
        output.push_str(&wire_name);
        output.push_str(" {\n");
        for value in &self.inputs.0 {
            output.push_str("        operation ");
            output.push_str(&value.operation);
            output.push('(');
            output.push_str(&value.payload);
            output.push_str("),\n");
        }
        output.push_str("    }\n    reply ");
        output.push_str(&reply_name);
        output.push_str(" {\n");
        for value in &self.outputs.0 {
            output.push_str("        ");
            output.push_str(&value.operation);
            output.push('(');
            output.push_str(&value.payload);
            output.push_str("),\n");
        }
        output.push_str("    }\n}\n\n");
        output.push_str("pub type ");
        output.push_str(&request_name);
        output.push_str(" = Operation;\n");
        Ok(output)
    }

    fn signal_rust_artifact(&self, path: PathBuf) -> Result<GeneratedArtifact, InterfaceFault> {
        Ok(GeneratedArtifact::new(path, self.signal_rust_source()?))
    }
}
