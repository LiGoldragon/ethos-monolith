//! Protosization: File to Protoform to Text (the ascent, cannot fault).
//!
//! Every concept yields the protoform that carries it, the exact
//! reverse of conception; the file's text is the canonical print of
//! its protoform, the braced form on one line.

use std::convert::Infallible;

use protos::{
    Bare, Delineation, Enclosure, Head, Protoform, Separator, Situated, Symbol, Textualizable,
};

use crate::{
    AssociatedConstant, AssociatedType, Association, Capability, Constraint, File, Identity,
    Import, Imported, KindBody, KindDeclaration, Named, Receiver, Reference, Rooted, Signature,
    Source, TypeDeclaration, Variant,
};

/// The kind whose capability yields the protoform carrying a concept.
trait Protosizing {
    fn protoform(&self) -> Protoform;
}

/// The kind whose capability yields the head naming a concept.
trait Heading {
    fn head(&self) -> Head;
}

/// The kind whose capability encloses the protoforms of every element.
trait Enclosing {
    fn bracketed(&self) -> Protoform;
    fn braced(&self) -> Protoform;
}

impl<P: Protosizing> Enclosing for [P] {
    fn bracketed(&self) -> Protoform {
        let mut children = Vec::with_capacity(self.len());
        for element in self {
            children.push(element.protoform());
        }
        Protoform::Enclosed(Enclosure::Bracketed, children)
    }

    fn braced(&self) -> Protoform {
        let mut children = Vec::with_capacity(self.len());
        for element in self {
            children.push(element.protoform());
        }
        Protoform::Enclosed(Enclosure::Braced, children)
    }
}

/// The kind whose capability heads a body with a name and a separator.
trait Attaching {
    fn under(self, head: Head, separator: Separator) -> Protoform;
}

/// The kind whose capability prefixes a body with every segment of a source.
trait Sourcing {
    fn under_source(self, source: &Source) -> Protoform;
}

/// The kind whose capabilities form validated Protos names and bare forms.
trait Naming {
    fn symbol(&self) -> Symbol;
    fn bare(&self) -> Protoform;
}

impl Naming for str {
    fn symbol(&self) -> Symbol {
        Symbol::try_from(self).expect("validated ethos name")
    }

    fn bare(&self) -> Protoform {
        Protoform::Bare(Bare::try_from(self).expect("validated ethos bare"))
    }
}

impl Attaching for Protoform {
    fn under(self, head: Head, separator: Separator) -> Protoform {
        Protoform::Headed(head, separator, Box::new(self))
    }
}

impl Sourcing for Protoform {
    fn under_source(self, source: &Source) -> Protoform {
        let mut headed = self;
        for segment in source.segments.iter().rev() {
            headed = headed.under(Head::Symbol(segment.clone()), Separator::Colon);
        }
        headed
    }
}

impl Protosizing for File {
    fn protoform(&self) -> Protoform {
        let sections = match self {
            File::Types(types) => vec![
                types.imports.bracketed(),
                types.types.bracketed(),
                types.associations.bracketed(),
            ],
            File::Kinds(kinds) => vec![kinds.imports.bracketed(), kinds.kinds.bracketed()],
            File::Signal(signal) => vec![
                signal.imports.bracketed(),
                signal.requests.bracketed(),
                signal.responses.bracketed(),
                signal.types.bracketed(),
            ],
            File::Sema(sema) => vec![
                sema.imports.bracketed(),
                sema.record.braced(),
                sema.types.bracketed(),
            ],
        };
        Protoform::Enclosed(Enclosure::Braced, sections)
            .under(Head::Symbol(self.root().name().symbol()), Separator::Period)
    }
}

impl Protosizing for Imported {
    fn protoform(&self) -> Protoform {
        if self.name == self.emitted {
            self.name.0.as_str().bare()
        } else {
            self.emitted.0.as_str().bare().under(
                Head::Symbol(self.name.0.as_str().symbol()),
                Separator::Period,
            )
        }
    }
}

impl Protosizing for Import {
    fn protoform(&self) -> Protoform {
        match self {
            Import::One(source, imported) => imported.protoform().under_source(source),
            Import::Many(source, imports) => imports.bracketed().under_source(source),
        }
    }
}

impl Heading for Reference {
    fn head(&self) -> Head {
        if self.arguments.is_empty() {
            Head::Symbol(self.name.0.as_str().symbol())
        } else {
            let mut arguments = Vec::with_capacity(self.arguments.len());
            for argument in &self.arguments {
                arguments.push(argument.protoform());
            }
            Head::Qualified(self.name.0.as_str().symbol(), arguments)
        }
    }
}

impl Protosizing for Reference {
    fn protoform(&self) -> Protoform {
        let bare = if self.arguments.is_empty() {
            self.name.0.as_str().bare()
        } else {
            let mut arguments = Vec::with_capacity(self.arguments.len());
            for argument in &self.arguments {
                arguments.push(argument.protoform());
            }
            Protoform::Qualified(self.name.0.as_str().symbol(), arguments)
        };
        match &self.source {
            Some(source) => bare.under_source(source),
            None => bare,
        }
    }
}

impl Heading for Identity {
    fn head(&self) -> Head {
        if self.constraints.is_empty() {
            Head::Symbol(self.name.0.as_str().symbol())
        } else {
            let mut constraints = Vec::with_capacity(self.constraints.len());
            for constraint in &self.constraints {
                constraints.push(constraint.protoform());
            }
            Head::Qualified(self.name.0.as_str().symbol(), constraints)
        }
    }
}

impl Protosizing for Constraint {
    fn protoform(&self) -> Protoform {
        match self {
            Constraint::One(reference) => reference.protoform(),
            Constraint::Many(references) => references.bracketed(),
        }
    }
}

impl Protosizing for TypeDeclaration {
    fn protoform(&self) -> Protoform {
        match self {
            TypeDeclaration::Struct(identity, positions) => {
                positions.braced().under(identity.head(), Separator::Period)
            }
            TypeDeclaration::Enum(identity, variants) => variants
                .bracketed()
                .under(identity.head(), Separator::Period),
            TypeDeclaration::Alias(identity, aliased) => aliased
                .protoform()
                .under(identity.head(), Separator::Period),
        }
    }
}

impl Protosizing for Variant {
    fn protoform(&self) -> Protoform {
        match self {
            Variant::Bare(name) => name.0.as_str().bare(),
            Variant::Typed(name, reference) => reference
                .protoform()
                .under(Head::Symbol(name.0.as_str().symbol()), Separator::Period),
            Variant::Struct(name, positions) => positions
                .braced()
                .under(Head::Symbol(name.0.as_str().symbol()), Separator::Period),
            Variant::Enum(name, variants) => variants
                .bracketed()
                .under(Head::Symbol(name.0.as_str().symbol()), Separator::Period),
        }
    }
}

impl Protosizing for KindDeclaration {
    fn protoform(&self) -> Protoform {
        let body = match &self.body {
            KindBody::Simple(capabilities) => capabilities.bracketed(),
            KindBody::Complex {
                superkinds,
                types,
                constants,
                capabilities,
            } => Protoform::Enclosed(
                Enclosure::Braced,
                vec![
                    superkinds.bracketed(),
                    types.bracketed(),
                    constants.bracketed(),
                    capabilities.bracketed(),
                ],
            ),
        };
        body.under(self.identity.head(), Separator::Period)
    }
}

impl Protosizing for AssociatedType {
    fn protoform(&self) -> Protoform {
        if self.bounds.is_empty() {
            self.name.0.as_str().bare()
        } else {
            let mut bounds = Vec::with_capacity(self.bounds.len());
            for bound in &self.bounds {
                bounds.push(bound.protoform());
            }
            Protoform::Qualified(self.name.0.as_str().symbol(), bounds)
        }
    }
}

impl Protosizing for AssociatedConstant {
    fn protoform(&self) -> Protoform {
        self.ty.protoform().under(
            Head::Symbol(self.name.0.as_str().symbol()),
            Separator::Period,
        )
    }
}

impl Protosizing for Capability {
    fn protoform(&self) -> Protoform {
        let separator = match self.receiver {
            Receiver::Shared => Separator::Period,
            Receiver::Mutable => Separator::Exclamation,
            Receiver::Static => Separator::Colon,
        };
        let body = match &self.signature {
            Signature::Yielding(yields) => std::slice::from_ref(yields).bracketed(),
            Signature::Taking(inputs, yields) => Protoform::Enclosed(
                Enclosure::Braced,
                vec![inputs.bracketed(), std::slice::from_ref(yields).bracketed()],
            ),
        };
        body.under(Head::Symbol(self.name.0.as_str().symbol()), separator)
    }
}

impl Protosizing for Association {
    fn protoform(&self) -> Protoform {
        self.kinds
            .bracketed()
            .under(self.identity.head(), Separator::Period)
    }
}

impl Textualizable for File {
    fn textualize(&self) -> String {
        self.protoform().textualize()
    }
}

/// A file directly projects the structural form it constructs. Situation is
/// computed while that form is written; ascent never reparses text.
impl protos::Protosizable for File {
    type Fault = Infallible;

    fn protosize(&self) -> Result<Delineation, Self::Fault> {
        let protoform = self.protoform();
        let situated = protos::Situating::situate(&protoform);
        Ok(Delineation(vec![Situated(situated.0, protoform)]))
    }
}
