//! Conception: Protoform to File (the reader, may fault).
//!
//! Every concept is conceived from the protoform that carries it, by a
//! `Conceiving<Concept>` interaction on `Protoform` (or on
//! `Head`, for what a head carries). Each interaction raises its faults
//! at paths relative to its own protoform; the container places them
//! under the child's index. A read file is checked whole before it is
//! yielded.

use protos::{Bare, Enclosure, Head, Protoform, Separator, Situated, Symbol};

use crate::checking::Checkable;

/// The kind whose capability conceives a concept from a protoform.
pub(crate) trait Conceiving<C> {
    fn conceive(&self) -> Result<C, Fault>;
}
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
    fn headed(&self) -> Option<Headed<'_>>;
    fn bare(&self) -> Option<&Bare>;
}

/// The three named parts of a headed protoform.
struct Headed<'a> {
    head: &'a Head,
    separator: Separator,
    body: &'a Protoform,
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

    fn headed(&self) -> Option<Headed<'_>> {
        match self {
            Protoform::Headed(head, separator, body) => Some(Headed {
                head,
                separator: *separator,
                body,
            }),
            _ => None,
        }
    }

    fn bare(&self) -> Option<&Bare> {
        match self {
            Protoform::Bare(head) => Some(head),
            _ => None,
        }
    }
}

trait Worded {
    fn text(&self) -> &str;
}
impl Worded for Symbol {
    fn text(&self) -> &str {
        self.as_ref()
    }
}
impl Worded for Bare {
    fn text(&self) -> &str {
        self.as_ref()
    }
}

/// The kind whose capabilities conceive every child of an enclosure, each placed under its index.
trait Enumerating {
    fn bracketed_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C>;
    fn braced_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C>;
    fn each<C>(children: &[Self]) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C> + Sized;
}

impl Enumerating for Protoform {
    fn bracketed_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C>,
    {
        match self.bracketed() {
            Some(children) => Self::each(children),
            None => Err(Problem::Expected(form).here()),
        }
    }

    fn braced_of<C>(&self, form: Form) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C>,
    {
        match self.braced() {
            Some(children) => Self::each(children),
            None => Err(Problem::Expected(form).here()),
        }
    }

    fn each<C>(children: &[Self]) -> Result<Vec<C>, Fault>
    where
        Self: Conceiving<C>,
    {
        let mut concepts = Vec::with_capacity(children.len());
        for (index, child) in children.iter().enumerate() {
            concepts.push(Conceiving::<C>::conceive(child).place(index as protos::Integer)?);
        }
        Ok(concepts)
    }
}

// ---------------------------------------------------------------------------
// Depth: the reader recurses, so the structure is bounded first, iteratively
// ---------------------------------------------------------------------------

/// How deep a structure may nest before the reader refuses it.
const DEPTH_LIMIT: usize = 128;

/// One pending structural visit in the bounded reader walk.
struct Pending<'a> {
    protoform: &'a Protoform,
    depth: usize,
    path: Vec<protos::Integer>,
}

/// The kind whose capability finds the first structure nested past the limit, walking with an explicit stack.
trait Bounded {
    fn bounded(&self, limit: usize) -> Result<(), Fault>;
}

impl Bounded for Protoform {
    fn bounded(&self, limit: usize) -> Result<(), Fault> {
        let mut pending = vec![Pending {
            protoform: self,
            depth: 0,
            path: vec![],
        }];
        while let Some(Pending {
            protoform,
            depth,
            path,
        }) = pending.pop()
        {
            if depth > limit {
                return Err(Fault::Conceptual(path, Problem::Depth));
            }
            match protoform {
                Protoform::Headed(head, _, body) => {
                    if let Head::Qualified(_, arguments) = head {
                        for (index, argument) in arguments.iter().enumerate() {
                            let mut child_path = path.clone();
                            child_path.push(0);
                            child_path.push(index as protos::Integer);
                            pending.push(Pending {
                                protoform: argument,
                                depth: depth + 1,
                                path: child_path,
                            });
                        }
                    }
                    let mut child_path = path.clone();
                    child_path.push(1);
                    pending.push(Pending {
                        protoform: body,
                        depth: depth + 1,
                        path: child_path,
                    });
                }
                Protoform::Enclosed(_, enclosed) => {
                    for (index, child) in enclosed.iter().enumerate() {
                        let mut child_path = path.clone();
                        child_path.push(index as protos::Integer);
                        pending.push(Pending {
                            protoform: child,
                            depth: depth + 1,
                            path: child_path,
                        });
                    }
                }
                Protoform::Qualified(_, arguments) => {
                    for (index, argument) in arguments.iter().enumerate() {
                        let mut child_path = path.clone();
                        child_path.push(index as protos::Integer);
                        pending.push(Pending {
                            protoform: argument,
                            depth: depth + 1,
                            path: child_path,
                        });
                    }
                }
                Protoform::Bare(_) | Protoform::Quoted(_) | Protoform::Parenthesized(_) => {}
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Names and sources: validated text
// ---------------------------------------------------------------------------

impl Conceiving<Name> for str {
    fn conceive(&self) -> Result<Name, Fault> {
        if !self.starts_with("r#") && (self == "Self" || syn::parse_str::<syn::Ident>(self).is_ok())
        {
            Ok(Name(self.to_owned()))
        } else {
            Err(Problem::Name(protos::Text::try_from(self).unwrap_or_default()).here())
        }
    }
}

impl Conceiving<Source> for str {
    fn conceive(&self) -> Result<Source, Fault> {
        let Ok(path) = syn::parse_str::<syn::Path>(self) else {
            return Err(Problem::Name(protos::Text::try_from(self).unwrap_or_default()).here());
        };
        if path.leading_colon.is_some() {
            return Err(Problem::Name(protos::Text::try_from(self).unwrap_or_default()).here());
        }
        for segment in &path.segments {
            if !segment.arguments.is_none() {
                return Err(Problem::Name(protos::Text::try_from(self).unwrap_or_default()).here());
            }
        }
        Ok(Source(self.to_owned()))
    }
}

// ---------------------------------------------------------------------------
// The file: a headed brace under one of the four roots
// ---------------------------------------------------------------------------

impl Conceiving<File> for protos::Delineation {
    fn conceive(&self) -> Result<File, Fault> {
        match self.0.as_slice() {
            [Situated(_, protoform)] => Conceiving::<File>::conceive(protoform).place(0),
            _ => Err(Problem::Root.here()),
        }
    }
}

/// The universal concept-layer interaction: a delineation conceives the one
/// situated Ethos file it carries.  The private reader interactions below are
/// its declaration anatomy, not an alternative public layer kind.
impl protos::Conceivable<File> for protos::Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<Situated<File>, Self::Fault> {
        match self.0.as_slice() {
            [Situated(situation, protoform)] => Conceiving::<File>::conceive(protoform)
                .map(|file| Situated(situation.clone(), file)),
            _ => Err(Problem::Root.here()),
        }
    }
}

impl Conceiving<File> for Protoform {
    fn conceive(&self) -> Result<File, Fault> {
        self.bounded(DEPTH_LIMIT)?;
        let Some(Headed {
            head,
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Root.here());
        };
        let Head::Symbol(symbol) = head else {
            return Err(Problem::Root.here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let Some(root) = Root::identify(symbol.text()) else {
            return Err(Problem::Root.here());
        };
        let file = match root {
            Root::Types => File::Types(Conceiving::<Types>::conceive(body).place(1)?),
            Root::Kinds => File::Kinds(Conceiving::<Kinds>::conceive(body).place(1)?),
            Root::Signal => File::Signal(Conceiving::<Signal>::conceive(body).place(1)?),
            Root::Sema => File::Sema(Conceiving::<Sema>::conceive(body).place(1)?),
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

impl Conceiving<Types> for Protoform {
    fn conceive(&self) -> Result<Types, Fault> {
        let sections = self.sections(3)?;
        Ok(Types {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            types: sections[1].bracketed_of(Form::Section).place(1)?,
            associations: sections[2].bracketed_of(Form::Section).place(2)?,
        })
    }
}

impl Conceiving<Kinds> for Protoform {
    fn conceive(&self) -> Result<Kinds, Fault> {
        let sections = self.sections(2)?;
        Ok(Kinds {
            imports: sections[0].bracketed_of(Form::Section).place(0)?,
            kinds: sections[1].bracketed_of(Form::Section).place(1)?,
        })
    }
}

impl Conceiving<Signal> for Protoform {
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

impl Conceiving<Sema> for Protoform {
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

impl Conceiving<Import> for Protoform {
    fn conceive(&self) -> Result<Import, Fault> {
        // A source:name import. Protos keeps every colon in a source chain as
        // a headed body, so each hop is structural and located precisely.
        if let Some(Headed {
            head: Head::Symbol(symbol),
            separator,
            body,
        }) = self.headed()
        {
            if separator != Separator::Colon {
                return Err(Problem::Separator(separator).here());
            }
            if let Some(Headed {
                head: Head::Symbol(segment),
                separator: Separator::Colon,
                body: names,
            }) = body.headed()
            {
                let source: Source = format!("{}::{}", symbol.text(), segment.text()).conceive()?;
                return match names.bracketed() {
                    Some(children) => Ok(Import::Many(
                        source,
                        Protoform::each(children).place(1).place(1)?,
                    )),
                    None => Ok(Import::One(
                        source,
                        Conceiving::<Imported>::conceive(names).place(1).place(1)?,
                    )),
                };
            }
            let source: Source = symbol.text().conceive()?;
            return match body.bracketed() {
                Some(children) => Ok(Import::Many(source, Protoform::each(children).place(1)?)),
                None => Ok(Import::One(
                    source,
                    Conceiving::<Imported>::conceive(body).place(1)?,
                )),
            };
        }
        if let Some(word) = self.bare()
            && let Some(colon) = word.text().rfind(':')
        {
            let source: Source = word.text()[..colon].conceive()?;
            let imported = Conceiving::<Imported>::conceive(&word.text()[colon + 1..])?;
            return Ok(Import::One(source, imported));
        }
        Err(Problem::Expected(Form::Import).here())
    }
}

impl Conceiving<Imported> for str {
    fn conceive(&self) -> Result<Imported, Fault> {
        if let Some(dot) = self.find('.') {
            let ethos_name = &self[..dot];
            let source_name = &self[dot + 1..];
            Ok(Imported {
                name: Conceiving::<Name>::conceive(ethos_name)?,
                emitted: Conceiving::<Name>::conceive(source_name).place(0)?,
            })
        } else {
            let name: Name = Conceiving::<Name>::conceive(self)?;
            Ok(Imported {
                emitted: name.clone(),
                name,
            })
        }
    }
}

impl Conceiving<Imported> for Protoform {
    fn conceive(&self) -> Result<Imported, Fault> {
        if let Some(symbol) = self.bare() {
            return Conceiving::<Imported>::conceive(symbol.text());
        }
        if let Some(Headed {
            head: Head::Symbol(symbol),
            separator: Separator::Period,
            body,
        }) = self.headed()
            && let Some(emitted) = body.bare()
        {
            return Ok(Imported {
                name: symbol.text().conceive()?,
                emitted: Conceiving::<Name>::conceive(emitted.text()).place(0)?,
            });
        }
        Err(Problem::Expected(Form::Import).here())
    }
}

// ---------------------------------------------------------------------------
// References, identities, constraints
// ---------------------------------------------------------------------------

impl Conceiving<Reference> for Head {
    fn conceive(&self) -> Result<Reference, Fault> {
        match self {
            Head::Symbol(symbol) => Ok(Reference {
                source: None,
                name: symbol.text().conceive()?,
                arguments: vec![],
            }),
            Head::Qualified(symbol, arguments) => Ok(Reference {
                source: None,
                name: symbol.text().conceive()?,
                arguments: Protoform::each(arguments)?,
            }),
        }
    }
}

impl Conceiving<Reference> for Protoform {
    fn conceive(&self) -> Result<Reference, Fault> {
        if let Some(head) = self.bare() {
            return Ok(Reference {
                source: None,
                name: head.text().conceive()?,
                arguments: vec![],
            });
        }
        if let Protoform::Qualified(symbol, arguments) = self {
            return Ok(Reference {
                source: None,
                name: symbol.text().conceive()?,
                arguments: Protoform::each(arguments)?,
            });
        }
        if let Some(Headed {
            head: Head::Symbol(symbol),
            separator: Separator::Colon,
            body,
        }) = self.headed()
        {
            let source: Source = symbol.text().conceive()?;
            let Some(head) = body.bare() else {
                return Err(Fault::Conceptual(
                    vec![1],
                    Problem::Expected(Form::Reference),
                ));
            };
            let reference = Reference {
                source: None,
                name: head.text().conceive().place(1)?,
                arguments: vec![],
            };
            return Ok(Reference {
                source: Some(source),
                ..reference
            });
        }
        Err(Problem::Expected(Form::Reference).here())
    }
}

impl Conceiving<Identity> for Head {
    fn conceive(&self) -> Result<Identity, Fault> {
        match self {
            Head::Symbol(symbol) if symbol.text() != "Self" => Ok(Identity {
                name: symbol.text().conceive()?,
                constraints: vec![],
            }),
            Head::Qualified(symbol, constraints) if symbol.text() != "Self" => Ok(Identity {
                name: symbol.text().conceive()?,
                constraints: Protoform::each(constraints)?,
            }),
            _ => Err(Problem::Name(protos::Text::try_from("Self").expect("static text")).here()),
        }
    }
}

impl Conceiving<Constraint> for Protoform {
    fn conceive(&self) -> Result<Constraint, Fault> {
        match self.bracketed() {
            Some(children) => Ok(Constraint::Many(Protoform::each(children)?)),
            None => Ok(Constraint::One(Conceiving::<Reference>::conceive(self)?)),
        }
    }
}

// ---------------------------------------------------------------------------
// Type declarations and variants
// ---------------------------------------------------------------------------

impl Conceiving<TypeDeclaration> for Protoform {
    fn conceive(&self) -> Result<TypeDeclaration, Fault> {
        let Some(Headed {
            head,
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Declaration).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let identity: Identity = head.conceive().place(0)?;
        if let Some(positions) = body.braced() {
            return Ok(TypeDeclaration::Struct(
                identity,
                Protoform::each(positions).place(1)?,
            ));
        }
        if let Some(variants) = body.bracketed() {
            return Ok(TypeDeclaration::Enum(
                identity,
                Protoform::each(variants).place(1)?,
            ));
        }
        Ok(TypeDeclaration::Alias(
            identity,
            Conceiving::<Reference>::conceive(body).place(1)?,
        ))
    }
}

impl Conceiving<Variant> for Protoform {
    fn conceive(&self) -> Result<Variant, Fault> {
        if let Some(symbol) = self.bare() {
            return Ok(Variant::Bare(symbol.text().conceive()?));
        }
        let Some(Headed {
            head: Head::Symbol(symbol),
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Variant).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let name: Name = symbol.text().conceive()?;
        if let Some(positions) = body.braced() {
            return Ok(Variant::Struct(name, Protoform::each(positions).place(1)?));
        }
        if let Some(variants) = body.bracketed() {
            return Ok(Variant::Enum(name, Protoform::each(variants).place(1)?));
        }
        Ok(Variant::Typed(
            name,
            Conceiving::<Reference>::conceive(body).place(1)?,
        ))
    }
}

// ---------------------------------------------------------------------------
// Kind declarations
// ---------------------------------------------------------------------------

impl Conceiving<KindDeclaration> for Protoform {
    fn conceive(&self) -> Result<KindDeclaration, Fault> {
        let Some(Headed {
            head,
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Kind).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        let identity: Identity = head.conceive().place(0)?;
        if body.bracketed().is_some() {
            return Ok(KindDeclaration {
                identity,
                body: KindBody::Simple(body.bracketed_of(Form::Kind).place(1)?),
            });
        }
        let Some(sections) = body.braced() else {
            return Err(Fault::Conceptual(vec![1], Problem::Expected(Form::Kind)));
        };
        let [superkinds, types, constants, capabilities] = sections else {
            return Err(Fault::Conceptual(
                vec![1],
                Problem::Arity(4, sections.len() as protos::Integer),
            ));
        };
        Ok(KindDeclaration {
            identity,
            body: KindBody::Complex {
                superkinds: superkinds
                    .bracketed_of(Form::Constraint)
                    .place(0)
                    .place(1)?,
                types: types.bracketed_of(Form::Kind).place(1).place(1)?,
                constants: constants.bracketed_of(Form::Constant).place(2).place(1)?,
                capabilities: capabilities
                    .bracketed_of(Form::Capability)
                    .place(3)
                    .place(1)?,
            },
        })
    }
}

impl Conceiving<AssociatedType> for Protoform {
    fn conceive(&self) -> Result<AssociatedType, Fault> {
        match self {
            Protoform::Bare(symbol) => Ok(AssociatedType {
                name: symbol.text().conceive()?,
                bounds: vec![],
            }),
            Protoform::Qualified(symbol, bounds) => Ok(AssociatedType {
                name: symbol.text().conceive()?,
                bounds: Protoform::each(bounds)?,
            }),
            _ => Err(Problem::Expected(Form::Kind).here()),
        }
    }
}

impl Conceiving<AssociatedConstant> for Protoform {
    fn conceive(&self) -> Result<AssociatedConstant, Fault> {
        let Some(Headed {
            head: Head::Symbol(symbol),
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Constant).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        Ok(AssociatedConstant {
            name: symbol.text().conceive()?,
            ty: Conceiving::<Reference>::conceive(body).place(1)?,
        })
    }
}

impl Conceiving<Receiver> for Separator {
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
            [one] => Conceiving::<Reference>::conceive(one).place(0),
            many => Err(Problem::Arity(1, many.len() as protos::Integer).here()),
        }
    }
}

impl Conceiving<Capability> for Protoform {
    fn conceive(&self) -> Result<Capability, Fault> {
        let Some(Headed {
            head: Head::Symbol(symbol),
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Capability).here());
        };
        let name: Name = symbol.text().conceive()?;
        let receiver: Receiver = separator.conceive()?;
        if body.bracketed().is_some() {
            return Ok(Capability {
                name,
                receiver,
                signature: Signature::Yielding(body.yields().place(1)?),
            });
        }
        let Some(sections) = body.braced() else {
            return Err(Fault::Conceptual(
                vec![1],
                Problem::Expected(Form::Capability),
            ));
        };
        let [inputs, yields] = sections else {
            return Err(Fault::Conceptual(
                vec![1],
                Problem::Arity(2, sections.len() as protos::Integer),
            ));
        };
        Ok(Capability {
            name,
            receiver,
            signature: Signature::Taking(
                inputs.bracketed_of(Form::Capability).place(0).place(1)?,
                yields.yields().place(1).place(1)?,
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Associations
// ---------------------------------------------------------------------------

impl Conceiving<Association> for Protoform {
    fn conceive(&self) -> Result<Association, Fault> {
        let Some(Headed {
            head,
            separator,
            body,
        }) = self.headed()
        else {
            return Err(Problem::Expected(Form::Association).here());
        };
        if separator != Separator::Period {
            return Err(Problem::Separator(separator).here());
        }
        Ok(Association {
            identity: head.conceive().place(0)?,
            kinds: body.bracketed_of(Form::Association).place(1)?,
        })
    }
}
