//! Conception: Protoform to File (the reader, may fault).
//!
//! Every concept is conceived from the protoform that carries it, by a
//! `protos::Conceivable<Concept>` interaction on `Protoform` (or on
//! `Head`, for what a head carries). Each interaction raises its faults
//! at paths relative to its own protoform; the container places them
//! under the child's index. A read file is checked whole before it is
//! yielded.

use protos::{Conceivable, Enclosure, Head, Protoform, Separator};

use crate::checking::Checkable;
use crate::{
    AssociatedConstant, AssociatedType, Association, Capability, Constraint, Fault, File, Form,
    Identifiable, Identity, Import, Imported, KindBody, KindDeclaration, Kinds, Name, Placing,
    Problem, Receiver, Reference, Root, Scope, Sema, Signal, Signature, Source, TypeDeclaration,
    Types, Variant,
};

// ---------------------------------------------------------------------------
// Faults at the protoform at hand
// ---------------------------------------------------------------------------

/// The kind whose capabilities raise a conceptual fault here, at the empty path.
trait Faulting {
    fn here(self) -> Fault;
}

impl Faulting for Problem {
    fn here(self) -> Fault {
        Fault::Conceptual(vec![], self)
    }
}

// ---------------------------------------------------------------------------
// Anatomy: what a protoform is, asked structurally
// ---------------------------------------------------------------------------

/// The kind whose capabilities expose a protoform's anatomy.
trait Anatomical {
    fn braced(&self) -> Option<&[Protoform]>;
    fn bracketed(&self) -> Option<&[Protoform]>;
    fn headed(&self) -> Option<(&Head, Separator, &Protoform)>;
    fn bare(&self) -> Option<&Head>;
}

impl Anatomical for Protoform {
    fn braced(&self) -> Option<&[Protoform]> {
        match self {
            Protoform::Enclosed(Enclosure::Braced, children) => Some(children),
            _ => None,
        }
    }

    fn bracketed(&self) -> Option<&[Protoform]> {
        match self {
            Protoform::Enclosed(Enclosure::Bracketed, children) => Some(children),
            _ => None,
        }
    }

    fn headed(&self) -> Option<(&Head, Separator, &Protoform)> {
        match self {
            Protoform::Headed(head, separator, body) => Some((head, *separator, body)),
            _ => None,
        }
    }

    fn bare(&self) -> Option<&Head> {
        match self {
            Protoform::Bare(head) => Some(head),
            _ => None,
        }
    }
}

/// The kind whose capabilities conceive every child of an enclosure, each placed under its index.
trait Enumerating {
    fn bracketed_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault>;
    fn braced_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault>;
    fn each<C>(children: &[Self]) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault> + Sized;
}

impl Enumerating for Protoform {
    fn bracketed_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault>,
    {
        match self.bracketed() {
            Some(children) => Self::each(children),
            None => Err(Problem::Expected(form).here()),
        }
    }

    fn braced_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault>,
    {
        match self.braced() {
            Some(children) => Self::each(children),
            None => Err(Problem::Expected(form).here()),
        }
    }

    fn each<C>(children: &[Self]) -> Result<Vec<C>, Fault>
    where
        Self: Conceivable<C, Fault = Fault>,
    {
        let mut concepts = Vec::with_capacity(children.len());
        for (index, child) in children.iter().enumerate() {
            concepts.push(Conceivable::<C>::conceive(child).place(index as protos::Integer)?);
        }
        Ok(concepts)
    }
}

// ---------------------------------------------------------------------------
// Depth: the reader recurses, so the structure is bounded first, iteratively
// ---------------------------------------------------------------------------

/// How deep a structure may nest before the reader refuses it.
const DEPTH_LIMIT: usize = 128;

/// The kind whose capability finds the first structure nested past the limit, walking with an explicit stack.
trait Bounded {
    fn bounded(&self, limit: usize) -> Result<(), Fault>;
}

impl Bounded for Protoform {
    fn bounded(&self, limit: usize) -> Result<(), Fault> {
        let mut pending: Vec<(&Protoform, usize, Vec<protos::Integer>)> = vec![(self, 0, vec![])];
        while let Some((protoform, depth, path)) = pending.pop() {
            if depth > limit {
                return Err(Fault::Conceptual(path, Problem::Depth));
            }
            let mut children: Vec<(&Protoform, protos::Integer)> = Vec::new();
            match protoform {
                Protoform::Headed(head, _, body) => {
                    if let Head::Qualified(_, arguments) = head {
                        for (index, argument) in arguments.iter().enumerate() {
                            children.push((argument, index as protos::Integer));
                        }
                    }
                    children.push((body, 0));
                }
                Protoform::Enclosed(_, enclosed) => {
                    for (index, child) in enclosed.iter().enumerate() {
                        children.push((child, index as protos::Integer));
                    }
                }
                Protoform::Bare(Head::Qualified(_, arguments)) => {
                    for (index, argument) in arguments.iter().enumerate() {
                        children.push((argument, index as protos::Integer));
                    }
                }
                Protoform::Bare(Head::Bare(_)) | Protoform::Opaque(_, _) => {}
            }
            for (child, index) in children {
                let mut child_path = path.clone();
                child_path.push(index);
                pending.push((child, depth + 1, child_path));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Names and sources: validated text
// ---------------------------------------------------------------------------

impl Conceivable<Name> for str {
    type Fault = Fault;

    fn conceive(&self) -> Result<Name, Fault> {
        if self == "Self" || syn::parse_str::<syn::Ident>(self).is_ok() {
            Ok(Name(self.to_owned()))
        } else {
            Err(Problem::Name(self.to_owned()).here())
        }
    }
}

impl Conceivable<Source> for str {
    type Fault = Fault;

    fn conceive(&self) -> Result<Source, Fault> {
        let Ok(path) = syn::parse_str::<syn::Path>(self) else {
            return Err(Problem::Name(self.to_owned()).here());
        };
        if path.leading_colon.is_some() {
            return Err(Problem::Name(self.to_owned()).here());
        }
        for segment in &path.segments {
            if !segment.arguments.is_none() {
                return Err(Problem::Name(self.to_owned()).here());
            }
        }
        Ok(Source(self.to_owned()))
    }
}

// ---------------------------------------------------------------------------
// The file: a headed brace under one of the four roots
// ---------------------------------------------------------------------------

impl Conceivable<File> for protos::Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<File, Fault> {
        match self.protoforms.as_slice() {
            [protoform] => Conceivable::<File>::conceive(protoform).place(0),
            _ => Err(Problem::Root.here()),
        }
    }
}

impl Conceivable<File> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<File, Fault> {
        self.bounded(DEPTH_LIMIT)?;
        let Some((head, separator, body)) = self.headed() else {
            return Err(Problem::Root.here());
        };
        let Head::Bare(symbol) = head else {
            return Err(Problem::Root.here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let Some(root) = Root::identify(symbol) else {
            return Err(Problem::Root.here());
        };
        let file = match root {
            Root::Types => File::Types(Conceivable::<Types>::conceive(body).place(0)?),
            Root::Kinds => File::Kinds(Conceivable::<Kinds>::conceive(body).place(0)?),
            Root::Signal => File::Signal(Conceivable::<Signal>::conceive(body).place(0)?),
            Root::Sema => File::Sema(Conceivable::<Sema>::conceive(body).place(0)?),
        };
        let scope = Scope {
            file: &file,
            identity: None,
            associated: &[],
        };
        file.check(&scope)?;
        Ok(file)
    }
}

/// The kind whose capability yields the sections of a braced body, exactly as many as the variant has.
trait Sectioning {
    fn sections(&self, count: usize) -> Result<&[Protoform], Fault>;
}

impl Sectioning for Protoform {
    fn sections(&self, count: usize) -> Result<&[Protoform], Fault> {
        let Some(sections) = self.braced() else {
            return Err(Problem::Expected(Form::File).here());
        };
        if sections.len() != count {
            return Err(Problem::Arity(
                count as protos::Integer,
                sections.len() as protos::Integer,
            )
            .here());
        }
        Ok(sections)
    }
}

impl Conceivable<Types> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Types, Fault> {
        let sections = self.sections(3)?;
        Ok(Types {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            types: sections[1].bracketed_of(Form::Section).place(1)?,
            associations: sections[2].bracketed_of(Form::Section).place(2)?,
        })
    }
}

impl Conceivable<Kinds> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Kinds, Fault> {
        let sections = self.sections(2)?;
        Ok(Kinds {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            kinds: sections[1].bracketed_of(Form::Section).place(1)?,
        })
    }
}

impl Conceivable<Signal> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Signal, Fault> {
        let sections = self.sections(4)?;
        Ok(Signal {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            requests: sections[1].bracketed_of(Form::Section).place(1)?,
            responses: sections[2].bracketed_of(Form::Section).place(2)?,
            types: sections[3].bracketed_of(Form::Section).place(3)?,
        })
    }
}

impl Conceivable<Sema> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Sema, Fault> {
        let sections = self.sections(3)?;
        Ok(Sema {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            record: sections[1].braced_of(Form::Section).place(1)?,
            types: sections[2].bracketed_of(Form::Section).place(2)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------------

impl Conceivable<Import> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Import, Fault> {
        let Some((Head::Bare(symbol), separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Import).here());
        };
        if separator != Separator::Colon {
            return Err(Problem::Separator(separator).here());
        }
        let source: Source = symbol.as_str().conceive()?;
        match body.bracketed() {
            Some(children) => Ok(Import::Many(source, Protoform::each(children).place(0)?)),
            None => Ok(Import::One(
                source,
                Conceivable::<Imported>::conceive(body).place(0)?,
            )),
        }
    }
}

impl Conceivable<Imported> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Imported, Fault> {
        if let Some(Head::Bare(symbol)) = self.bare() {
            let name: Name = symbol.as_str().conceive()?;
            return Ok(Imported {
                emitted: name.clone(),
                name,
            });
        }
        if let Some((Head::Bare(symbol), Separator::Period, body)) = self.headed()
            && let Some(Head::Bare(emitted)) = body.bare()
        {
            return Ok(Imported {
                name: symbol.as_str().conceive()?,
                emitted: Conceivable::<Name>::conceive(emitted.as_str()).place(0)?,
            });
        }
        Err(Problem::Expected(Form::Import).here())
    }
}

// ---------------------------------------------------------------------------
// References, identities, constraints
// ---------------------------------------------------------------------------

impl Conceivable<Reference> for Head {
    type Fault = Fault;

    fn conceive(&self) -> Result<Reference, Fault> {
        match self {
            Head::Bare(symbol) => Ok(Reference {
                source: None,
                name: symbol.as_str().conceive()?,
                arguments: vec![],
            }),
            Head::Qualified(symbol, arguments) => Ok(Reference {
                source: None,
                name: symbol.as_str().conceive()?,
                arguments: Protoform::each(arguments)?,
            }),
        }
    }
}

impl Conceivable<Reference> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Reference, Fault> {
        if let Some(head) = self.bare() {
            return head.conceive();
        }
        if let Some((Head::Bare(symbol), Separator::Colon, body)) = self.headed() {
            let source: Source = symbol.as_str().conceive()?;
            let Some(head) = body.bare() else {
                return Err(Fault::Conceptual(
                    vec![0],
                    Problem::Expected(Form::Reference),
                ));
            };
            let reference: Reference = Conceivable::<Reference>::conceive(head).place(0)?;
            return Ok(Reference {
                source: Some(source),
                ..reference
            });
        }
        Err(Problem::Expected(Form::Reference).here())
    }
}

impl Conceivable<Identity> for Head {
    type Fault = Fault;

    fn conceive(&self) -> Result<Identity, Fault> {
        match self {
            Head::Bare(symbol) => Ok(Identity {
                name: symbol.as_str().conceive()?,
                constraints: vec![],
            }),
            Head::Qualified(symbol, constraints) => Ok(Identity {
                name: symbol.as_str().conceive()?,
                constraints: Protoform::each(constraints)?,
            }),
        }
    }
}

impl Conceivable<Constraint> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Constraint, Fault> {
        match self.bracketed() {
            Some(children) => Ok(Constraint::Many(Protoform::each(children)?)),
            None => Ok(Constraint::One(Conceivable::<Reference>::conceive(self)?)),
        }
    }
}

// ---------------------------------------------------------------------------
// Type declarations and variants
// ---------------------------------------------------------------------------

impl Conceivable<TypeDeclaration> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<TypeDeclaration, Fault> {
        let Some((head, separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Declaration).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let identity: Identity = head.conceive()?;
        if let Some(positions) = body.braced() {
            return Ok(TypeDeclaration::Struct(
                identity,
                Protoform::each(positions).place(0)?,
            ));
        }
        if let Some(variants) = body.bracketed() {
            return Ok(TypeDeclaration::Enum(
                identity,
                Protoform::each(variants).place(0)?,
            ));
        }
        Ok(TypeDeclaration::Alias(
            identity,
            Conceivable::<Reference>::conceive(body).place(0)?,
        ))
    }
}

impl Conceivable<Variant> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Variant, Fault> {
        if let Some(Head::Bare(symbol)) = self.bare() {
            return Ok(Variant::Bare(symbol.as_str().conceive()?));
        }
        let Some((Head::Bare(symbol), separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Variant).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let name: Name = symbol.as_str().conceive()?;
        if let Some(positions) = body.braced() {
            return Ok(Variant::Struct(name, Protoform::each(positions).place(0)?));
        }
        if let Some(variants) = body.bracketed() {
            return Ok(Variant::Enum(name, Protoform::each(variants).place(0)?));
        }
        Ok(Variant::Typed(
            name,
            Conceivable::<Reference>::conceive(body).place(0)?,
        ))
    }
}

// ---------------------------------------------------------------------------
// Kind declarations
// ---------------------------------------------------------------------------

impl Conceivable<KindDeclaration> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<KindDeclaration, Fault> {
        let Some((head, separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Kind).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let identity: Identity = head.conceive()?;
        if body.bracketed().is_some() {
            return Ok(KindDeclaration {
                identity,
                body: KindBody::Simple(body.bracketed_of(Form::Kind).place(0)?),
            });
        }
        let Some(sections) = body.braced() else {
            return Err(Fault::Conceptual(vec![0], Problem::Expected(Form::Kind)));
        };
        let [superkinds, types, constants, capabilities] = sections else {
            return Err(Fault::Conceptual(
                vec![0],
                Problem::Arity(4, sections.len() as protos::Integer),
            ));
        };
        Ok(KindDeclaration {
            identity,
            body: KindBody::Complex {
                superkinds: superkinds
                    .bracketed_of(Form::Constraint)
                    .place(0)
                    .place(0)?,
                types: types.bracketed_of(Form::Kind).place(1).place(0)?,
                constants: constants.bracketed_of(Form::Constant).place(2).place(0)?,
                capabilities: capabilities
                    .bracketed_of(Form::Capability)
                    .place(3)
                    .place(0)?,
            },
        })
    }
}

impl Conceivable<AssociatedType> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<AssociatedType, Fault> {
        match self.bare() {
            Some(Head::Bare(symbol)) => Ok(AssociatedType {
                name: symbol.as_str().conceive()?,
                bounds: vec![],
            }),
            Some(Head::Qualified(symbol, bounds)) => Ok(AssociatedType {
                name: symbol.as_str().conceive()?,
                bounds: Protoform::each(bounds)?,
            }),
            None => Err(Problem::Expected(Form::Kind).here()),
        }
    }
}

impl Conceivable<AssociatedConstant> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<AssociatedConstant, Fault> {
        let Some((Head::Bare(symbol), separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Constant).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        Ok(AssociatedConstant {
            name: symbol.as_str().conceive()?,
            ty: Conceivable::<Reference>::conceive(body).place(0)?,
        })
    }
}

impl Conceivable<Receiver> for Separator {
    type Fault = Fault;

    fn conceive(&self) -> Result<Receiver, Fault> {
        Ok(match self {
            Separator::Period => Receiver::Shared,
            Separator::Exclamation => Receiver::Mutable,
            Separator::Colon => Receiver::Static,
        })
    }
}

/// The kind whose capability reads a yield bracket: exactly one type.
trait Yielding {
    fn yields(&self) -> Result<Reference, Fault>;
}

impl Yielding for Protoform {
    fn yields(&self) -> Result<Reference, Fault> {
        let Some(children) = self.bracketed() else {
            return Err(Problem::Expected(Form::Capability).here());
        };
        match children {
            [] => Err(Problem::Yield.here()),
            [one] => Conceivable::<Reference>::conceive(one).place(0),
            many => Err(Problem::Arity(1, many.len() as protos::Integer).here()),
        }
    }
}

impl Conceivable<Capability> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Capability, Fault> {
        let Some((Head::Bare(symbol), separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Capability).here());
        };
        let name: Name = symbol.as_str().conceive()?;
        let receiver: Receiver = separator.conceive()?;
        if body.bracketed().is_some() {
            return Ok(Capability {
                name,
                receiver,
                signature: Signature::Yielding(body.yields().place(0)?),
            });
        }
        let Some(sections) = body.braced() else {
            return Err(Fault::Conceptual(
                vec![0],
                Problem::Expected(Form::Capability),
            ));
        };
        let [inputs, yields] = sections else {
            return Err(Fault::Conceptual(
                vec![0],
                Problem::Arity(2, sections.len() as protos::Integer),
            ));
        };
        Ok(Capability {
            name,
            receiver,
            signature: Signature::Taking(
                inputs.bracketed_of(Form::Capability).place(0).place(0)?,
                yields.yields().place(1).place(0)?,
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Associations
// ---------------------------------------------------------------------------

impl Conceivable<Association> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<Association, Fault> {
        let Some((head, separator, body)) = self.headed() else {
            return Err(Problem::Expected(Form::Association).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        Ok(Association {
            identity: head.conceive()?,
            kinds: body.bracketed_of(Form::Association).place(0)?,
        })
    }
}
