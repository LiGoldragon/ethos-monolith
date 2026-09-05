//! Protosization: File to Protoform to Text (the ascent, cannot fault).
//!
//! Every concept yields the protoform that carries it, the exact
//! reverse of conception; the file's text is the canonical print of
//! its protoform, the braced form on one line.

use protos::{Enclosure, Head, Protoform, Separator, Textualizable};

use crate::{
    AssociatedConstant, AssociatedType, Association, Capability, Constraint, File, Identity,
    Import, Imported, KindBody, KindDeclaration, Named, Receiver, Reference, Rooted, Signature,
    TypeDeclaration, Variant,
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

impl Attaching for Protoform {
    fn under(self, head: Head, separator: Separator) -> Protoform {
        Protoform::Headed(head, separator, Box::new(self))
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
        Protoform::Enclosed(Enclosure::Braced, sections).under(
            Head::Symbol(self.root().name().to_owned()),
            Separator::Period,
        )
    }
}

impl Protosizing for Imported {
    fn protoform(&self) -> Protoform {
        if self.name == self.emitted {
            Protoform::Bare(Head::Symbol(self.name.0.to_string()))
        } else {
            Protoform::Bare(Head::Symbol(self.emitted.0.to_string()))
                .under(Head::Symbol(self.name.0.to_string()), Separator::Period)
        }
    }
}

impl Protosizing for Import {
    fn protoform(&self) -> Protoform {
        match self {
            Import::One(source, imported) => imported
                .protoform()
                .under(Head::Symbol(source.0.to_string()), Separator::Colon),
            Import::Many(source, imports) => imports
                .bracketed()
                .under(Head::Symbol(source.0.to_string()), Separator::Colon),
        }
    }
}

impl Heading for Reference {
    fn head(&self) -> Head {
        if self.arguments.is_empty() {
            Head::Symbol(self.name.0.to_string())
        } else {
            let mut arguments = Vec::with_capacity(self.arguments.len());
            for argument in &self.arguments {
                arguments.push(argument.protoform());
            }
            Head::Qualified(self.name.0.to_string(), arguments)
        }
    }
}

impl Protosizing for Reference {
    fn protoform(&self) -> Protoform {
        let bare = Protoform::Bare(self.head());
        match &self.source {
            Some(source) => bare.under(Head::Symbol(source.0.to_string()), Separator::Colon),
            None => bare,
        }
    }
}

impl Heading for Identity {
    fn head(&self) -> Head {
        if self.constraints.is_empty() {
            Head::Symbol(self.name.0.to_string())
        } else {
            let mut constraints = Vec::with_capacity(self.constraints.len());
            for constraint in &self.constraints {
                constraints.push(constraint.protoform());
            }
            Head::Qualified(self.name.0.to_string(), constraints)
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
            Variant::Bare(name) => Protoform::Bare(Head::Symbol(name.0.to_string())),
            Variant::Typed(name, reference) => reference
                .protoform()
                .under(Head::Symbol(name.0.to_string()), Separator::Period),
            Variant::Struct(name, positions) => positions
                .braced()
                .under(Head::Symbol(name.0.to_string()), Separator::Period),
            Variant::Enum(name, variants) => variants
                .bracketed()
                .under(Head::Symbol(name.0.to_string()), Separator::Period),
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
            Protoform::Bare(Head::Symbol(self.name.0.to_string()))
        } else {
            let mut bounds = Vec::with_capacity(self.bounds.len());
            for bound in &self.bounds {
                bounds.push(bound.protoform());
            }
            Protoform::Bare(Head::Qualified(self.name.0.to_string(), bounds))
        }
    }
}

impl Protosizing for AssociatedConstant {
    fn protoform(&self) -> Protoform {
        self.ty
            .protoform()
            .under(Head::Symbol(self.name.0.to_string()), Separator::Period)
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
        body.under(Head::Symbol(self.name.0.to_string()), separator)
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
