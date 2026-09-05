//! Checking: the file validated whole (may fault).
//!
//! Resolution is borne by the declarations: an import resolves the
//! names it carries, a declaration its own name, an identity its
//! parameters, a kind its associated types, and the file walks its
//! variant's sections in turn, then the intrinsics. Checking walks the
//! concept as the protoform was laid out, so every fault is at the
//! path of the structure at fault, relative to the checked value.

use protos::{Integer, Path};

use crate::{
    AssociatedConstant, AssociatedType, Association, Capability, Constraint, Fault, File,
    Identifiable, Identity, Import, Intrinsic, KindBody, KindDeclaration, Kinds, Name, Placing,
    Problem, Reference, Resolution, Resolving, Role, Scope, Sema, Signal, Signature,
    TypeDeclaration, Types, Variant,
};

/// A schema must remain small enough for complete whole-file checking to have
/// a caller-visible, finite cost.  Deep structure has a separate reader
/// bound; this bounds the flat declaration graph.
const DECLARATION_LIMIT: usize = 512;

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

impl Resolving for Import {
    fn resolve(&self, name: &Name) -> Resolution {
        match self {
            Import::One(source, imported) if &imported.name == name => {
                Resolution::Imported(source.clone(), imported.emitted.clone())
            }
            Import::One(_, _) => Resolution::Undeclared,
            Import::Many(source, imports) => {
                for imported in imports {
                    if &imported.name == name {
                        return Resolution::Imported(source.clone(), imported.emitted.clone());
                    }
                }
                Resolution::Undeclared
            }
        }
    }
}

impl Resolving for [Import] {
    fn resolve(&self, name: &Name) -> Resolution {
        for import in self {
            let resolution = import.resolve(name);
            if resolution != Resolution::Undeclared {
                return resolution;
            }
        }
        Resolution::Undeclared
    }
}

impl Resolving for TypeDeclaration {
    fn resolve(&self, name: &Name) -> Resolution {
        let declared = match self {
            TypeDeclaration::Struct(identity, _)
            | TypeDeclaration::Enum(identity, _)
            | TypeDeclaration::Alias(identity, _) => &identity.name,
        };
        if declared == name {
            Resolution::Type(name.clone())
        } else {
            Resolution::Undeclared
        }
    }
}

impl Resolving for [TypeDeclaration] {
    fn resolve(&self, name: &Name) -> Resolution {
        for declaration in self {
            let resolution = declaration.resolve(name);
            if resolution != Resolution::Undeclared {
                return resolution;
            }
        }
        Resolution::Undeclared
    }
}

impl Resolving for KindDeclaration {
    fn resolve(&self, name: &Name) -> Resolution {
        if &self.identity.name == name {
            Resolution::Kind(name.clone())
        } else {
            Resolution::Undeclared
        }
    }
}

impl Resolving for [KindDeclaration] {
    fn resolve(&self, name: &Name) -> Resolution {
        for declaration in self {
            let resolution = declaration.resolve(name);
            if resolution != Resolution::Undeclared {
                return resolution;
            }
        }
        Resolution::Undeclared
    }
}

/// The kind whose capability yields the names a file variant implies: its query, response or record type.
pub(crate) trait Implying {
    /// The implied type names.
    fn implied(&self) -> Vec<Name>;
}

impl Implying for File {
    fn implied(&self) -> Vec<Name> {
        match self {
            File::Types(_) | File::Kinds(_) => vec![],
            File::Signal(_) => vec![Name("Request".to_owned()), Name("Response".to_owned())],
            File::Sema(_) => vec![Name("Record".to_owned())],
        }
    }
}

impl Resolving for File {
    fn resolve(&self, name: &Name) -> Resolution {
        let resolution = match self {
            File::Types(types) => types.types.resolve(name).or(types.imports.resolve(name)),
            File::Kinds(kinds) => kinds.kinds.resolve(name).or(kinds.imports.resolve(name)),
            File::Signal(signal) => signal.types.resolve(name).or(signal.imports.resolve(name)),
            File::Sema(sema) => sema.types.resolve(name).or(sema.imports.resolve(name)),
        };
        if resolution != Resolution::Undeclared {
            return resolution;
        }
        if self.implied().contains(name) {
            return Resolution::Type(name.clone());
        }
        match Intrinsic::identify(&name.0) {
            Some(intrinsic) => Resolution::Intrinsic(intrinsic),
            None => Resolution::Undeclared,
        }
    }
}

/// The kind whose capability yields the first resolution that is not undeclared.
trait Falling {
    fn or(self, other: Resolution) -> Resolution;
}

impl Falling for Resolution {
    fn or(self, other: Resolution) -> Resolution {
        if self == Resolution::Undeclared {
            other
        } else {
            self
        }
    }
}

impl Resolving for Identity {
    fn resolve(&self, name: &Name) -> Resolution {
        for (index, constraint) in self.constraints.iter().enumerate() {
            if let Constraint::One(reference) = constraint
                && reference.source.is_none()
                && reference.arguments.is_empty()
                && &reference.name == name
            {
                return Resolution::Parameter(index as Integer);
            }
        }
        Resolution::Undeclared
    }
}

impl Resolving for [AssociatedType] {
    fn resolve(&self, name: &Name) -> Resolution {
        for associated in self {
            if &associated.name == name {
                return Resolution::Associated(name.clone());
            }
        }
        Resolution::Undeclared
    }
}

impl Resolving for Scope<'_> {
    fn resolve(&self, name: &Name) -> Resolution {
        let parameter = match self.identity {
            Some(identity) => identity.resolve(name),
            None => Resolution::Undeclared,
        };
        parameter
            .or(self.associated.resolve(name))
            .or(self.file.resolve(name))
    }
}

// ---------------------------------------------------------------------------
// Intrinsic arity
// ---------------------------------------------------------------------------

/// The kind whose capability yields how many arguments an intrinsic takes.
pub(crate) trait Taking {
    /// The argument count.
    fn arity(&self) -> usize;
}

impl Taking for Intrinsic {
    fn arity(&self) -> usize {
        match self {
            Intrinsic::Vector | Intrinsic::Option => 1,
            Intrinsic::Result => 2,
            Intrinsic::Text
            | Intrinsic::Integer
            | Intrinsic::Decimal
            | Intrinsic::Boolean
            | Intrinsic::Meaning
            | Intrinsic::Itself
            | Intrinsic::Sized => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Naming: every name a value declares, with the path it was declared at
// ---------------------------------------------------------------------------

/// The kind whose capability lists the names a value declares, each at its path relative to the value.
pub(crate) trait Naming {
    /// The declared names and their paths.
    fn names(&self) -> Vec<(Name, Path)>;
}

impl Naming for Import {
    fn names(&self) -> Vec<(Name, Path)> {
        match self {
            Import::One(_, imported) => vec![(imported.name.clone(), vec![0])],
            Import::Many(_, imports) => {
                let mut names = Vec::with_capacity(imports.len());
                for (index, imported) in imports.iter().enumerate() {
                    names.push((imported.name.clone(), vec![0, index as Integer]));
                }
                names
            }
        }
    }
}

impl Naming for TypeDeclaration {
    fn names(&self) -> Vec<(Name, Path)> {
        match self {
            TypeDeclaration::Struct(identity, _)
            | TypeDeclaration::Enum(identity, _)
            | TypeDeclaration::Alias(identity, _) => vec![(identity.name.clone(), vec![])],
        }
    }
}

impl Naming for KindDeclaration {
    fn names(&self) -> Vec<(Name, Path)> {
        vec![(self.identity.name.clone(), vec![])]
    }
}

impl Naming for Variant {
    fn names(&self) -> Vec<(Name, Path)> {
        match self {
            Variant::Bare(name)
            | Variant::Typed(name, _)
            | Variant::Struct(name, _)
            | Variant::Enum(name, _) => vec![(name.clone(), vec![])],
        }
    }
}

impl Naming for AssociatedType {
    fn names(&self) -> Vec<(Name, Path)> {
        vec![(self.name.clone(), vec![])]
    }
}

impl Naming for AssociatedConstant {
    fn names(&self) -> Vec<(Name, Path)> {
        vec![(self.name.clone(), vec![])]
    }
}

impl Naming for Capability {
    fn names(&self) -> Vec<(Name, Path)> {
        vec![(self.name.clone(), vec![])]
    }
}

/// The kind whose capability lists the names of every element of a section, each under its index.
trait Sectioned {
    fn names_in(&self, section: Integer) -> Vec<(Name, Path)>;
}

impl<N: Naming> Sectioned for [N] {
    fn names_in(&self, section: Integer) -> Vec<(Name, Path)> {
        let mut names = Vec::new();
        for (index, element) in self.iter().enumerate() {
            for (name, path) in element.names() {
                let mut placed = vec![section, index as Integer];
                placed.extend(path);
                names.push((name, placed));
            }
        }
        names
    }
}

/// The kind whose capability faults on the second occurrence of a name.
trait Distinct {
    fn distinct(&self) -> Result<(), Fault>;
}

impl Distinct for [(Name, Path)] {
    fn distinct(&self) -> Result<(), Fault> {
        for (later, (name, path)) in self.iter().enumerate() {
            for (earlier, _) in &self[..later] {
                if earlier == name {
                    return Err(Fault::Conceptual(
                        path.clone(),
                        Problem::Duplicate(
                            protos::Text::try_from(name.0.clone()).expect("identifier"),
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Checking
// ---------------------------------------------------------------------------

/// The kind whose capability checks a value in a scope, faulting at a path relative to the value.
pub(crate) trait Checkable {
    /// Check the value whole.
    fn check(&self, scope: &Scope) -> Result<(), Fault>;
}

/// The kind whose capability checks every element of a section, each placed under its index.
trait Checking {
    fn check_each(&self, scope: &Scope, section: Integer) -> Result<(), Fault>;
}

impl<C: Checkable> Checking for [C] {
    fn check_each(&self, scope: &Scope, section: Integer) -> Result<(), Fault> {
        for (index, element) in self.iter().enumerate() {
            element
                .check(scope)
                .place(index as Integer)
                .place(section)?;
        }
        Ok(())
    }
}

impl Checkable for File {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        let checked = match self {
            File::Types(types) => types.check(scope),
            File::Kinds(kinds) => kinds.check(scope),
            File::Signal(signal) => signal.check(scope),
            File::Sema(sema) => sema.check(scope),
        };
        checked.place(0)
    }
}

impl Checkable for Types {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        if self.types.len() > DECLARATION_LIMIT {
            return Err(Fault::Conceptual(vec![1], Problem::Depth));
        }
        let mut names = self.imports.names_in(0);
        names.extend(self.types.names_in(1));
        names.distinct()?;
        self.types.check_each(scope, 1)?;
        self.associations.check_each(scope, 2)
    }
}

impl Checkable for Kinds {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        let mut names = self.imports.names_in(0);
        names.extend(self.kinds.names_in(1));
        names.distinct()?;
        self.kinds.check_each(scope, 1)
    }
}

impl Checkable for Signal {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        if self.requests.is_empty() {
            return Err(Fault::Conceptual(vec![1], Problem::Empty));
        }
        if self.responses.is_empty() {
            return Err(Fault::Conceptual(vec![2], Problem::Empty));
        }
        let mut names = vec![
            (Name("Request".to_owned()), vec![1]),
            (Name("Response".to_owned()), vec![2]),
        ];
        names.extend(self.imports.names_in(0));
        names.extend(self.types.names_in(3));
        names.distinct()?;
        self.requests.names_in(1).distinct()?;
        self.responses.names_in(2).distinct()?;
        self.requests.check_each(scope, 1)?;
        self.responses.check_each(scope, 2)?;
        self.types.check_each(scope, 3)
    }
}

impl Checkable for Sema {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        let mut names = vec![(Name("Record".to_owned()), vec![1])];
        names.extend(self.imports.names_in(0));
        names.extend(self.types.names_in(2));
        names.distinct()?;
        self.record.check_each(scope, 1)?;
        self.types.check_each(scope, 2)
    }
}

impl Checkable for Identity {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        if self.constraints.len() > 26 {
            return Err(Fault::Conceptual(
                vec![],
                Problem::Arity(26, self.constraints.len() as Integer),
            ));
        }
        for (index, constraint) in self.constraints.iter().enumerate() {
            constraint.check(scope).place(index as Integer)?;
        }
        Ok(())
    }
}

impl Checkable for Constraint {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        match self {
            Constraint::One(reference) => reference.refer(scope, Role::Kind),
            Constraint::Many(references) => {
                if references.is_empty() {
                    return Err(Fault::Conceptual(vec![], Problem::Empty));
                }
                for (index, reference) in references.iter().enumerate() {
                    reference.refer(scope, Role::Kind).place(index as Integer)?;
                }
                Ok(())
            }
        }
    }
}

/// The kind whose capability yields whether an intrinsic is a type or a kind.
trait Roled {
    fn role(&self) -> Role;
}

impl Roled for Intrinsic {
    fn role(&self) -> Role {
        match self {
            Intrinsic::Sized => Role::Kind,
            Intrinsic::Text
            | Intrinsic::Integer
            | Intrinsic::Decimal
            | Intrinsic::Boolean
            | Intrinsic::Meaning
            | Intrinsic::Vector
            | Intrinsic::Option
            | Intrinsic::Result
            | Intrinsic::Itself => Role::Type,
        }
    }
}

/// The kind whose capability checks a reference in the role its position gives it.
pub(crate) trait Referring {
    /// Check the reference as a type or as a kind.
    fn refer(&self, scope: &Scope, role: Role) -> Result<(), Fault>;
}

/// The kind whose capability checks every reference of a section in one role.
trait ReferringEach {
    fn refer_each(&self, scope: &Scope, role: Role, section: Integer) -> Result<(), Fault>;
}

impl ReferringEach for [Reference] {
    fn refer_each(&self, scope: &Scope, role: Role, section: Integer) -> Result<(), Fault> {
        for (index, reference) in self.iter().enumerate() {
            reference
                .refer(scope, role)
                .place(index as Integer)
                .place(section)?;
        }
        Ok(())
    }
}

impl Referring for Reference {
    fn refer(&self, scope: &Scope, role: Role) -> Result<(), Fault> {
        // A direct Protos intrinsic has the same contract whether it arrives
        // through an import or an explicit qualification. Other sources own
        // their declaration metadata.
        if self
            .source
            .as_ref()
            .is_some_and(|source| source.0 == "protos")
            && let Some(intrinsic) = Intrinsic::identify(&self.name.0)
        {
            if intrinsic.role() != role {
                return Err(Fault::Conceptual(
                    vec![],
                    Problem::Role(protos::Text::try_from(self.name.0.clone()).expect("identifier")),
                ));
            }
            if intrinsic.arity() != self.arguments.len() {
                return Err(Fault::Conceptual(
                    vec![],
                    Problem::Arity(
                        intrinsic.arity() as Integer,
                        self.arguments.len() as Integer,
                    ),
                ));
            }
        }
        // A sourced name is otherwise the source's to declare; its head sits at child 0.
        if self.source.is_none() {
            let (found, expected) = match scope.resolve(&self.name) {
                Resolution::Undeclared => {
                    return Err(Fault::Conceptual(
                        vec![],
                        Problem::Undeclared(
                            protos::Text::try_from(self.name.0.clone()).expect("identifier"),
                        ),
                    ));
                }
                Resolution::Intrinsic(intrinsic) => (intrinsic.role(), Some(intrinsic.arity())),
                Resolution::Parameter(_) | Resolution::Associated(_) => (Role::Type, Some(0)),
                Resolution::Type(name) => {
                    let arity =
                        scope
                            .file
                            .declaration(&name)
                            .map(|declaration| match declaration {
                                TypeDeclaration::Struct(identity, _)
                                | TypeDeclaration::Enum(identity, _)
                                | TypeDeclaration::Alias(identity, _) => identity.constraints.len(),
                            });
                    (Role::Type, arity)
                }
                Resolution::Kind(_) => (Role::Kind, None),
                Resolution::Imported(source, emitted)
                    if source.0 == "protos" && emitted == self.name =>
                {
                    match Intrinsic::identify(&self.name.0) {
                        Some(intrinsic) => (intrinsic.role(), Some(intrinsic.arity())),
                        None => (role, None),
                    }
                }
                Resolution::Imported(_, _) => (role, None),
            };
            if found != role {
                return Err(Fault::Conceptual(
                    vec![],
                    Problem::Role(protos::Text::try_from(self.name.0.clone()).expect("identifier")),
                ));
            }
            if let Some(expected) = expected
                && expected != self.arguments.len()
            {
                return Err(Fault::Conceptual(
                    vec![],
                    Problem::Arity(expected as Integer, self.arguments.len() as Integer),
                ));
            }
        }
        for (index, argument) in self.arguments.iter().enumerate() {
            let checked = argument.refer(scope, Role::Type).place(index as Integer);
            match self.source {
                Some(_) => checked.place(0)?,
                None => checked?,
            }
        }
        Ok(())
    }
}

impl Checkable for Reference {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        self.refer(scope, Role::Type)
    }
}

/// The kind whose capability tells whether an alias reaches a name through aliases and intrinsic containers alone.
trait Cycling {
    fn cycles(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool;
}

impl Cycling for Reference {
    fn cycles(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        if self.source.is_some() {
            return false;
        }
        if &self.name == target {
            return true;
        }
        match file.resolve(&self.name) {
            Resolution::Type(name) => {
                if visited.contains(&name) {
                    return false;
                }
                visited.push(name.clone());
                match file.declaration(&name) {
                    Some(TypeDeclaration::Alias(_, aliased)) => {
                        aliased.cycles(target, file, visited)
                    }
                    _ => false,
                }
            }
            Resolution::Intrinsic(_) | Resolution::Imported(_, _) => {
                for argument in &self.arguments {
                    if argument.cycles(target, file, visited) {
                        return true;
                    }
                }
                false
            }
            Resolution::Kind(_)
            | Resolution::Parameter(_)
            | Resolution::Associated(_)
            | Resolution::Undeclared => false,
        }
    }
}

/// The kind whose capability finds the declaration of a name in a file.
pub(crate) trait Declaring {
    /// The type declaration named, if the file declares one.
    fn declaration(&self, name: &Name) -> Option<&TypeDeclaration>;
}

impl Declaring for File {
    fn declaration(&self, name: &Name) -> Option<&TypeDeclaration> {
        let declarations = match self {
            File::Types(types) => &types.types,
            File::Kinds(_) => return None,
            File::Signal(signal) => &signal.types,
            File::Sema(sema) => &sema.types,
        };
        for declaration in declarations {
            if declaration.resolve(name) != Resolution::Undeclared {
                return Some(declaration);
            }
        }
        None
    }
}

impl Checkable for TypeDeclaration {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        let identity = match self {
            TypeDeclaration::Struct(identity, _)
            | TypeDeclaration::Enum(identity, _)
            | TypeDeclaration::Alias(identity, _) => identity,
        };
        identity.check(scope)?;
        let inner = Scope {
            file: scope.file,
            identity: Some(identity),
            associated: scope.associated,
        };
        match self {
            TypeDeclaration::Struct(_, positions) => positions.check_each(&inner, 0),
            TypeDeclaration::Enum(_, variants) => {
                variants.names_in(0).distinct()?;
                variants.check_each(&inner, 0)
            }
            TypeDeclaration::Alias(identity, aliased) => {
                aliased.check(&inner).place(0)?;
                if aliased.cycles(&identity.name, scope.file, &mut vec![]) {
                    return Err(Fault::Conceptual(
                        vec![0],
                        Problem::Cycle(
                            protos::Text::try_from(identity.name.0.clone()).expect("identifier"),
                        ),
                    ));
                }
                Ok(())
            }
        }
    }
}

impl Checkable for Variant {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        match self {
            Variant::Bare(_) => Ok(()),
            Variant::Typed(_, reference) => reference.check(scope).place(0),
            Variant::Struct(_, positions) => positions.check_each(scope, 0),
            Variant::Enum(_, variants) => {
                variants.names_in(0).distinct()?;
                variants.check_each(scope, 0)
            }
        }
    }
}

impl Checkable for KindDeclaration {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        self.identity.check(scope)?;
        let inner = Scope {
            file: scope.file,
            identity: Some(&self.identity),
            associated: match &self.body {
                KindBody::Simple(_) => &[],
                KindBody::Complex { types, .. } => types,
            },
        };
        match &self.body {
            KindBody::Simple(capabilities) => {
                capabilities.names_in(0).distinct()?;
                capabilities.check_each(&inner, 0)
            }
            KindBody::Complex {
                superkinds,
                types,
                constants,
                capabilities,
            } => {
                types.names_in(1).distinct()?;
                constants.names_in(2).distinct()?;
                capabilities.names_in(3).distinct()?;
                superkinds.refer_each(&inner, Role::Kind, 0).place(0)?;
                types.check_each(&inner, 1).place(0)?;
                constants.check_each(&inner, 2).place(0)?;
                capabilities.check_each(&inner, 3).place(0)
            }
        }
    }
}

impl Checkable for AssociatedType {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        for (index, bound) in self.bounds.iter().enumerate() {
            bound.refer(scope, Role::Kind).place(index as Integer)?;
        }
        Ok(())
    }
}

impl Checkable for AssociatedConstant {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        if self.name.0 != self.name.0.to_uppercase() {
            return Err(Fault::Conceptual(
                vec![],
                Problem::Name(protos::Text::try_from(self.name.0.clone()).expect("identifier")),
            ));
        }
        self.ty.check(scope).place(0)
    }
}

impl Checkable for Capability {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        match &self.signature {
            Signature::Yielding(yields) => yields.check(scope).place(0).place(0),
            Signature::Taking(inputs, yields) => {
                inputs.check_each(scope, 0).place(0)?;
                yields.check(scope).place(0).place(1).place(0)
            }
        }
    }
}

impl Checkable for Association {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        if scope.resolve(&self.identity.name) == Resolution::Undeclared {
            return Err(Fault::Conceptual(
                vec![],
                Problem::Undeclared(
                    protos::Text::try_from(self.identity.name.0.clone()).expect("identifier"),
                ),
            ));
        }
        self.identity.check(scope)?;
        // The kinds borne are named outside the identity that bears them.
        self.kinds.refer_each(scope, Role::Kind, 0)
    }
}
