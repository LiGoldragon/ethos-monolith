//! Ethos-zero: the ethos schema language, version zero.
//!
//! Ethos specifies the types, datom fills them with data, and ethos
//! generates the Rust. This crate reads an ethos file and generates
//! its Rust module. The layers, top to bottom, and the kind that
//! carries a value from one to the next:
//!
//! | layer | type | kind borne | yields |
//! |---|---|---|---|
//! | Text, as written (the sweet form) | `protos::Text` | [`Canonicalizable`] | [`Canonical`] |
//! | Text, canonical (the braced form) | [`Canonical`] | `protos::Protosizable` | `protos::Delineation` |
//! | Protoform | `protos::Delineation`, `protos::Protoform` | `Conceiving<File>` | [`File`], checked whole |
//! | Concept | [`File`] | [`Generating`] | Rust text |
//!
//! `protos::Potential<File>` bears `protos::Actualizable<File>`: the
//! whole descent in one call, its fault situated by path and extent in
//! the source text. The concept goes back up too: [`File`] bears
//! `protos::Protosizable` and `protos::Textualizable`, which cannot
//! fault.
//!
//! Every fault the reader raises is a [`Fault`] carrying the path of
//! the structure at fault, in Protos's path convention: a headed structure's
//! head is child 0 and its body is child 1; a qualified head's arguments are
//! children of that head; an enclosure's children are
//! numbered from 0, and each container prepends its child's index on
//! the way up (`protos::Pathed::within`).
//!
//! The generated Rust for a declared type has the shape datom-codec
//! gives its own intrinsics: `datom_codec::Datomic` with
//! `incorporate(site: Site<'_>)` and `conceive(&self) -> Datom`.

// A walk over the variants of an enum is written as the loop it is, not
// as an iterator adaptor with an inlined closure: no closure beyond what
// std forces, and no free function, is the crate's own rule.
#![allow(clippy::manual_find, clippy::manual_map)]

use protos::{Extent, Integer, Pathed};

// ---------------------------------------------------------------------------
// The faults: declared in fault.ethos, generated into fault.rs
// ---------------------------------------------------------------------------

#[rustfmt::skip]
mod fault;

pub use fault::{Fault, Form, Problem};

// ---------------------------------------------------------------------------
// The concept: the File and its declarations
// ---------------------------------------------------------------------------

/// A validated identifier: the name of a type, kind, variant, capability or constant.
///
/// Construct it with [`TryFrom<&str>`]; `AsRef<str>` reads its validated text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name(String);

/// The source of an import: a Rust path prefix such as `protos`, `crate` or `std::clone`.
///
/// Construct it with [`TryFrom<&str>`]; `AsRef<str>` reads its validated text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    text: String,
    segments: Vec<protos::Symbol>,
}

impl TryFrom<&str> for Name {
    type Error = String;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        if !text.starts_with("r#")
            && (text == "Self" || syn::parse_str::<syn::Ident>(text).is_ok())
            && protos::Symbol::try_from(text).is_ok()
        {
            Ok(Self(text.to_owned()))
        } else {
            Err(text.to_owned())
        }
    }
}

impl TryFrom<String> for Name {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::try_from(text.as_str()).map(|_| Self(text))
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Source {
    type Error = String;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let Ok(path) = syn::parse_str::<syn::Path>(text) else {
            return Err(text.to_owned());
        };
        if path.leading_colon.is_some() {
            return Err(text.to_owned());
        }
        for segment in &path.segments {
            if !segment.arguments.is_none() {
                return Err(text.to_owned());
            }
        }
        let mut segments = Vec::with_capacity(path.segments.len());
        for segment in text.split("::") {
            let Ok(symbol) = protos::Symbol::try_from(segment) else {
                return Err(text.to_owned());
            };
            segments.push(symbol);
        }
        Ok(Self {
            text: text.to_owned(),
            segments,
        })
    }
}

impl TryFrom<String> for Source {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::try_from(text.as_str())
    }
}

impl AsRef<str> for Source {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// The unit of declaration: one file, one Rust module; an enum of its four variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum File {
    /// Types: imports, type declarations, associations.
    Types(Types),
    /// Kinds: imports, kind declarations.
    Kinds(Kinds),
    /// Signal: imports, the query variants, the response variants, the types carried.
    Signal(Signal),
    /// Sema: imports, the record's positions, the types stored.
    Sema(Sema),
}

/// The head of a file: which variant of [`File`] it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Root {
    /// The types variant.
    Types,
    /// The kinds variant.
    Kinds,
    /// The signal variant.
    Signal,
    /// The sema variant.
    Sema,
}

/// The types variant of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Types {
    /// Where imported names come from.
    pub imports: Vec<Import>,
    /// The type declarations.
    pub types: Vec<TypeDeclaration>,
    /// Which types bear which kinds.
    pub associations: Vec<Association>,
}

/// The kinds variant of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kinds {
    /// Where imported names come from.
    pub imports: Vec<Import>,
    /// The kind declarations.
    pub kinds: Vec<KindDeclaration>,
}

/// The signal variant of a file: a wire contract whose query type is `Request` and whose response type is `Response`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    /// Where imported names come from.
    pub imports: Vec<Import>,
    /// The variants of the query type `Request`.
    pub requests: Vec<Variant>,
    /// The variants of the response type `Response`.
    pub responses: Vec<Variant>,
    /// The types the requests and responses carry.
    pub types: Vec<TypeDeclaration>,
}

/// The sema variant of a file: a storage contract whose record type is `Record`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sema {
    /// Where imported names come from.
    pub imports: Vec<Import>,
    /// The positions of the record type `Record`.
    pub record: Vec<Reference>,
    /// The types the record stores.
    pub types: Vec<TypeDeclaration>,
}

/// An import: a source and the names taken from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Import {
    /// One name from a source: `protos:Text`.
    One(Source, Imported),
    /// Several names from a source: `protos:[ Text Integer ]`.
    Many(Source, Vec<Imported>),
}

/// An imported name: the ethos name and the source's own name for it, the same unless written `Ethos.Source`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    /// The name as ethos writes it.
    pub name: Name,
    /// The name the source gives it, which the generated Rust writes.
    pub emitted: Name,
}

/// A reference to a type or a kind by name: an optional inline source, the name, and its arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    /// An inline source qualifying the name: `protos:Fault`.
    pub source: Option<Source>,
    /// The name referred to.
    pub name: Name,
    /// The arguments in angle brackets: `Vector<Text>`, `Result<Integer SinkError>`.
    pub arguments: Vec<Reference>,
}

/// The identity of a type or a kind: its name and its constraints, written as one head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    /// The name.
    pub name: Name,
    /// The constraints, one per parameter the Rust needs.
    pub constraints: Vec<Constraint>,
}

/// A constraint: a kind, or a bracket of kinds, bounding one parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Constraint {
    /// One kind: `Serializable`.
    One(Reference),
    /// A bracket of kinds: `[Clonable Sendable]`.
    Many(Vec<Reference>),
}

/// A type declaration: a struct of positions, an enum of variants, or an alias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDeclaration {
    /// A headed brace: the positions in order.
    Struct(Identity, Vec<Reference>),
    /// A headed bracket: the variants.
    Enum(Identity, Vec<Variant>),
    /// A headed bare: the aliased type.
    Alias(Identity, Reference),
}

/// A variant of an enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Carrying nothing: `Closed`.
    Bare(Name),
    /// Carrying one type: `Lock.LockRequest`.
    Typed(Name, Reference),
    /// Carrying an inline struct, a tuple variant: `Node.{ Tree Tree }`.
    Struct(Name, Vec<Reference>),
    /// Carrying an inline enum, a nested enum type: `Kind.[ A B ]`.
    Enum(Name, Vec<Variant>),
}

/// A kind declaration: the bearer of capabilities, a trait in the Rust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindDeclaration {
    /// Its identity: the name and the constraints.
    pub identity: Identity,
    /// Its definition.
    pub body: KindBody,
}

/// The definition of a kind: simple, a bracket of capabilities; or complex, a brace of four brackets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindBody {
    /// `Name.[ capabilities ]`.
    Simple(Vec<Capability>),
    /// `Name.{ [ superkinds ] [ associated types ] [ associated constants ] [ capabilities ] }`.
    Complex {
        /// The kinds it extends.
        superkinds: Vec<Reference>,
        /// Its associated types.
        types: Vec<AssociatedType>,
        /// Its associated constants.
        constants: Vec<AssociatedConstant>,
        /// Its capabilities.
        capabilities: Vec<Capability>,
    },
}

/// An associated type of a kind, with the kinds bounding it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssociatedType {
    /// The name.
    pub name: Name,
    /// The bounds: `Item<Serializable>`.
    pub bounds: Vec<Reference>,
}

/// An associated constant of a kind: its name and its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssociatedConstant {
    /// The upper-case name.
    pub name: Name,
    /// The type.
    pub ty: Reference,
}

/// A capability: a function a kind has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capability {
    /// The name.
    pub name: Name,
    /// Who is called.
    pub receiver: Receiver,
    /// What it takes and what it yields.
    pub signature: Signature,
}

/// A capability's signature: a yield bracket alone, or a brace of inputs and yield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signature {
    /// `name.[ Yield ]`.
    Yielding(Reference),
    /// `name.{ [ inputs ] [ Yield ] }`.
    Taking(Vec<Reference>, Reference),
}

/// Who a capability is called on, said by its separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receiver {
    /// `.` takes self.
    Shared,
    /// `!` takes mutable self.
    Mutable,
    /// `:` takes no self.
    Static,
}

/// An association: a type, by its identity, bears kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Association {
    /// The type's identity.
    pub identity: Identity,
    /// The kinds it bears.
    pub kinds: Vec<Reference>,
}

// ---------------------------------------------------------------------------
// The text layer: the canonical form
// ---------------------------------------------------------------------------

/// The canonical text of a file, and the seam where the sweet form was opened into braces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canonical {
    /// The braced form the reader sees.
    pub text: String,
    /// The bytes inserted after the head, empty when the text was already canonical.
    pub seam: Extent,
}

// ---------------------------------------------------------------------------
// Resolution: what a name names
// ---------------------------------------------------------------------------

/// The names known without import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intrinsic {
    /// `protos::Text`.
    Text,
    /// `protos::Integer`.
    Integer,
    /// `protos::Decimal`.
    Decimal,
    /// `protos::Boolean`.
    Boolean,
    /// `datom_codec::Meaning`.
    Meaning,
    /// `Vec`.
    Vector,
    /// `Option`.
    Option,
    /// `Result`.
    Result,
    /// `Self`.
    Itself,
    /// `Sized`, the bound every corporate type bears.
    Sized,
}

/// What a name resolves to in a scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// An intrinsic, written fully qualified.
    Intrinsic(Intrinsic),
    /// An imported name, written as the source's path and the source's name.
    Imported(Source, Name),
    /// A type declared in this file, written bare.
    Type(Name),
    /// A kind declared in this file, written bare.
    Kind(Name),
    /// The parameter bounded by the enclosing identity's constraint at this index.
    Parameter(Integer),
    /// More than one enclosing parameter has this name among its bounds.
    ///
    /// A body reference cannot say which parameter it means, even when the
    /// constraints differ as whole groups.
    Ambiguous(Name),
    /// An associated type of the enclosing kind, written `Self::Name`.
    Associated(Name),
    /// A name nothing declares.
    Undeclared,
}

/// What a reference is asked to be: a type in a type position, a kind in a bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A type: a position, an alias, an input, a yield, an argument.
    Type,
    /// A kind: a constraint, a superkind, a bound, an association.
    Kind,
}

/// The scope a reference resolves in: the file, and the identity and associated types of the enclosing declaration.
#[derive(Clone, Copy, Debug)]
pub struct Scope<'a> {
    /// The file whose imports and declarations are in scope.
    pub file: &'a File,
    /// The enclosing identity, whose single-kind constraints name parameters.
    pub identity: Option<&'a Identity>,
    /// The enclosing kind's associated types.
    pub associated: &'a [AssociatedType],
}

// ---------------------------------------------------------------------------
// Kinds
// ---------------------------------------------------------------------------

/// The kind whose capability yields the canonical form of an ethos text.
pub trait Canonicalizable {
    /// Open the sweet form into the braced form; the text is delineated to find its head.
    fn canonicalize(&self) -> Result<Canonical, protos::Fault>;
}

/// The kind whose capability maps an extent of the canonical text back onto the source text.
pub trait Resituating {
    /// Map an extent across the seam.
    fn resituate(&self, extent: Extent) -> Extent;
}

/// The kind whose capability yields the ethos name of a value.
pub trait Named {
    /// The name as ethos writes it.
    fn name(&self) -> &'static str;
}

/// The kind whose static capability identifies a variant from its ethos name, walking the variants.
pub trait Identifiable: Sized {
    /// Identify the variant named.
    fn identify(name: &str) -> Option<Self>;
}

/// The kind whose capability yields which variant of [`File`] a value is.
pub trait Rooted {
    /// The head the file is written under.
    fn root(&self) -> Root;
}

/// The kind whose capability resolves a name to what it names.
pub trait Resolving {
    /// Resolve a name.
    fn resolve(&self, name: &Name) -> Resolution;
}

/// The kind whose capability generates the Rust module of a file; it cannot fault, the file having been checked whole.
pub trait Generating {
    /// The Rust text, formatted.
    fn generate(&self) -> String;
}

/// The kind whose capability places a result's fault under a child index.
pub trait Placing {
    /// Prepend the index to the fault's path.
    fn place(self, index: Integer) -> Self;
}

// ---------------------------------------------------------------------------
// Fault interactions
// ---------------------------------------------------------------------------

impl Pathed for Fault {
    fn path(&self) -> &[Integer] {
        match self {
            Fault::Structural(_) => &[],
            Fault::Conceptual(path, _) => path,
        }
    }

    fn within(self, index: Integer) -> Self {
        match self {
            Fault::Structural(fault) => Fault::Structural(fault),
            Fault::Conceptual(mut path, problem) => {
                path.insert(0, index);
                Fault::Conceptual(path, problem)
            }
        }
    }
}

impl From<protos::Fault> for Fault {
    fn from(fault: protos::Fault) -> Self {
        Fault::Structural(fault)
    }
}

impl<T> Placing for Result<T, Fault> {
    fn place(self, index: Integer) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(fault) => Err(fault.within(index)),
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", protos::Textualizable::textualize(self))
    }
}

impl std::error::Error for Fault {}

// ---------------------------------------------------------------------------
// Root and Intrinsic: named, identified by walking the variants
// ---------------------------------------------------------------------------

impl Named for Root {
    fn name(&self) -> &'static str {
        match self {
            Root::Types => "Types",
            Root::Kinds => "Kinds",
            Root::Signal => "Signal",
            Root::Sema => "Sema",
        }
    }
}

impl Identifiable for Root {
    fn identify(name: &str) -> Option<Self> {
        for root in [Root::Types, Root::Kinds, Root::Signal, Root::Sema] {
            if root.name() == name {
                return Some(root);
            }
        }
        None
    }
}

impl Rooted for File {
    fn root(&self) -> Root {
        match self {
            File::Types(_) => Root::Types,
            File::Kinds(_) => Root::Kinds,
            File::Signal(_) => Root::Signal,
            File::Sema(_) => Root::Sema,
        }
    }
}

impl Named for Intrinsic {
    fn name(&self) -> &'static str {
        match self {
            Intrinsic::Text => "Text",
            Intrinsic::Integer => "Integer",
            Intrinsic::Decimal => "Decimal",
            Intrinsic::Boolean => "Boolean",
            Intrinsic::Meaning => "Meaning",
            Intrinsic::Vector => "Vector",
            Intrinsic::Option => "Option",
            Intrinsic::Result => "Result",
            Intrinsic::Itself => "Self",
            Intrinsic::Sized => "Sized",
        }
    }
}

impl Identifiable for Intrinsic {
    fn identify(name: &str) -> Option<Self> {
        for intrinsic in [
            Intrinsic::Text,
            Intrinsic::Integer,
            Intrinsic::Decimal,
            Intrinsic::Boolean,
            Intrinsic::Meaning,
            Intrinsic::Vector,
            Intrinsic::Option,
            Intrinsic::Result,
            Intrinsic::Itself,
            Intrinsic::Sized,
        ] {
            if intrinsic.name() == name {
                return Some(intrinsic);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Passes: implementation below, each module named for its pass
// ---------------------------------------------------------------------------

mod actualization;
mod canonicalization;
mod checking;
mod conception;
mod datomization;
mod generation;
mod protosization;
