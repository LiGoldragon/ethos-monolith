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
        let mut parameter = None;
        for (index, constraint) in self.constraints.iter().enumerate() {
            let references = match constraint {
                Constraint::One(reference) => std::slice::from_ref(reference),
                Constraint::Many(references) => references,
            };
            if references.iter().any(|reference| {
                reference.source.is_none()
                    && reference.arguments.is_empty()
                    && &reference.name == name
            }) {
                if parameter.is_some() {
                    return Resolution::Ambiguous(name.clone());
                }
                parameter = Some(index as Integer);
            }
        }
        match parameter {
            Some(index) => Resolution::Parameter(index),
            None => Resolution::Undeclared,
        }
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
trait Naming {
    /// The declared names and their paths.
    fn names(&self) -> Vec<DeclarationSite>;
}

/// A declared name and its structural path.
struct DeclarationSite {
    name: Name,
    path: Path,
}

impl Naming for Import {
    fn names(&self) -> Vec<DeclarationSite> {
        match self {
            Import::One(_, imported) => vec![DeclarationSite {
                name: imported.name.clone(),
                path: vec![1],
            }],
            Import::Many(_, imports) => {
                let mut names = Vec::with_capacity(imports.len());
                for (index, imported) in imports.iter().enumerate() {
                    names.push(DeclarationSite {
                        name: imported.name.clone(),
                        path: vec![1, index as Integer],
                    });
                }
                names
            }
        }
    }
}

impl Naming for TypeDeclaration {
    fn names(&self) -> Vec<DeclarationSite> {
        match self {
            TypeDeclaration::Struct(identity, _)
            | TypeDeclaration::Enum(identity, _)
            | TypeDeclaration::Alias(identity, _) => vec![DeclarationSite {
                name: identity.name.clone(),
                path: vec![0],
            }],
        }
    }
}

impl Naming for KindDeclaration {
    fn names(&self) -> Vec<DeclarationSite> {
        vec![DeclarationSite {
            name: self.identity.name.clone(),
            path: vec![0],
        }]
    }
}

impl Naming for Variant {
    fn names(&self) -> Vec<DeclarationSite> {
        match self {
            Variant::Bare(name) => vec![DeclarationSite {
                name: name.clone(),
                path: vec![],
            }],
            Variant::Typed(name, _) | Variant::Struct(name, _) | Variant::Enum(name, _) => {
                vec![DeclarationSite {
                    name: name.clone(),
                    path: vec![0],
                }]
            }
        }
    }
}

impl Naming for AssociatedType {
    fn names(&self) -> Vec<DeclarationSite> {
        vec![DeclarationSite {
            name: self.name.clone(),
            path: vec![],
        }]
    }
}

impl Naming for AssociatedConstant {
    fn names(&self) -> Vec<DeclarationSite> {
        vec![DeclarationSite {
            name: self.name.clone(),
            path: vec![0],
        }]
    }
}

impl Naming for Capability {
    fn names(&self) -> Vec<DeclarationSite> {
        vec![DeclarationSite {
            name: self.name.clone(),
            path: vec![0],
        }]
    }
}

/// The kind whose capability lists the names of every element of a section, each under its index.
trait Sectioned {
    fn names_in(&self, section: Integer) -> Vec<DeclarationSite>;
}

impl<N: Naming> Sectioned for [N] {
    fn names_in(&self, section: Integer) -> Vec<DeclarationSite> {
        let mut names = Vec::new();
        for (index, element) in self.iter().enumerate() {
            for declared in element.names() {
                let mut placed = vec![section, index as Integer];
                placed.extend(declared.path);
                names.push(DeclarationSite {
                    name: declared.name,
                    path: placed,
                });
            }
        }
        names
    }
}

/// The kind whose capability faults on the second occurrence of a name.
trait Distinct {
    fn distinct(&self) -> Result<(), Fault>;
}

impl Distinct for [DeclarationSite] {
    fn distinct(&self) -> Result<(), Fault> {
        for (later, declared) in self.iter().enumerate() {
            for earlier in &self[..later] {
                if earlier.name == declared.name {
                    return Err(Fault::Conceptual(
                        declared.path.clone(),
                        Problem::Duplicate(
                            protos::Text::try_from(declared.name.0.clone()).expect("identifier"),
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

/// The kind whose capabilities check enclosed children, or a section that
/// contains enclosed children.
trait Checking {
    fn check_children(&self, scope: &Scope) -> Result<(), Fault>;
    fn check_each(&self, scope: &Scope, section: Integer) -> Result<(), Fault>;
}

impl<C: Checkable> Checking for [C] {
    fn check_children(&self, scope: &Scope) -> Result<(), Fault> {
        for (index, element) in self.iter().enumerate() {
            element.check(scope).place(index as Integer)?;
        }
        Ok(())
    }

    fn check_each(&self, scope: &Scope, section: Integer) -> Result<(), Fault> {
        self.check_children(scope).place(section)
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
        checked.place(1)
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
            DeclarationSite {
                name: Name("Request".to_owned()),
                path: vec![1],
            },
            DeclarationSite {
                name: Name("Response".to_owned()),
                path: vec![2],
            },
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
        let mut names = vec![DeclarationSite {
            name: Name("Record".to_owned()),
            path: vec![1],
        }];
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
        let mut parameters: Vec<&Name> = Vec::new();
        for (index, constraint) in self.constraints.iter().enumerate() {
            if let Constraint::One(reference) = constraint
                && reference.source.is_none()
                && reference.arguments.is_empty()
            {
                if parameters.contains(&&reference.name) {
                    return Err(Fault::Conceptual(
                        vec![index as Integer],
                        Problem::Duplicate(
                            protos::Text::try_from(reference.name.0.clone()).expect("identifier"),
                        ),
                    ));
                }
                parameters.push(&reference.name);
            }
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

/// The role and optional argument count a resolved reference requires.
struct ReferenceRequirement {
    role: Role,
    arity: Option<usize>,
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
        // A sourced reference carries the foreign name in the headed body's
        // structural child at index one.
        if self.source.is_none() {
            let requirement = match scope.resolve(&self.name) {
                Resolution::Ambiguous(name) => {
                    return Err(Fault::Conceptual(
                        vec![],
                        Problem::Duplicate(protos::Text::try_from(name.0).expect("identifier")),
                    ));
                }
                Resolution::Undeclared => {
                    return Err(Fault::Conceptual(
                        vec![],
                        Problem::Undeclared(
                            protos::Text::try_from(self.name.0.clone()).expect("identifier"),
                        ),
                    ));
                }
                Resolution::Intrinsic(intrinsic) => ReferenceRequirement {
                    role: intrinsic.role(),
                    arity: Some(intrinsic.arity()),
                },
                Resolution::Parameter(_) | Resolution::Associated(_) => ReferenceRequirement {
                    role: Role::Type,
                    arity: Some(0),
                },
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
                    ReferenceRequirement {
                        role: Role::Type,
                        arity,
                    }
                }
                Resolution::Kind(_) => ReferenceRequirement {
                    role: Role::Kind,
                    arity: None,
                },
                Resolution::Imported(source, emitted)
                    if source.0 == "protos" && emitted == self.name =>
                {
                    match Intrinsic::identify(&self.name.0) {
                        Some(intrinsic) => ReferenceRequirement {
                            role: intrinsic.role(),
                            arity: Some(intrinsic.arity()),
                        },
                        None => ReferenceRequirement { role, arity: None },
                    }
                }
                Resolution::Imported(_, _) => ReferenceRequirement { role, arity: None },
            };
            if requirement.role != role {
                return Err(Fault::Conceptual(
                    vec![],
                    Problem::Role(protos::Text::try_from(self.name.0.clone()).expect("identifier")),
                ));
            }
            if let Some(expected) = requirement.arity
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
                Some(_) => checked.place(1)?,
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
            | Resolution::Ambiguous(_)
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

/// The kind whose capability finds a kind declaration in a kinds file.
trait KindDeclaring {
    fn kind_declaration(&self, name: &Name) -> Option<&KindDeclaration>;
}

impl KindDeclaring for File {
    fn kind_declaration(&self, name: &Name) -> Option<&KindDeclaration> {
        let File::Kinds(kinds) = self else {
            return None;
        };
        kinds
            .kinds
            .iter()
            .find(|declaration| declaration.identity.name == *name)
    }
}

/// The kind whose capability finds an indirect superkind cycle.
trait Supercycling {
    fn reaches_superkind(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool;
}

impl Supercycling for Reference {
    fn reaches_superkind(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        if self.source.is_some() {
            return false;
        }
        if &self.name == target {
            return true;
        }
        if visited.contains(&self.name) {
            return false;
        }
        let Some(declaration) = file.kind_declaration(&self.name) else {
            return false;
        };
        let KindBody::Complex { superkinds, .. } = &declaration.body else {
            return false;
        };
        visited.push(self.name.clone());
        for superkind in superkinds {
            if superkind.reaches_superkind(target, file, visited) {
                return true;
            }
        }
        false
    }
}

impl Checkable for TypeDeclaration {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        let identity = match self {
            TypeDeclaration::Struct(identity, _)
            | TypeDeclaration::Enum(identity, _)
            | TypeDeclaration::Alias(identity, _) => identity,
        };
        identity.check(scope).place(0)?;
        let inner = Scope {
            file: scope.file,
            identity: Some(identity),
            associated: scope.associated,
        };
        match self {
            TypeDeclaration::Struct(_, positions) => positions.check_children(&inner).place(1),
            TypeDeclaration::Enum(_, variants) => {
                let mut names = variants.names_in(0);
                for declared in &mut names {
                    declared.path.remove(0);
                    declared.path.insert(0, 1);
                }
                names.distinct()?;
                variants.check_children(&inner).place(1)
            }
            TypeDeclaration::Alias(identity, aliased) => {
                aliased.check(&inner).place(1)?;
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
            Variant::Typed(_, reference) => reference.check(scope).place(1),
            Variant::Struct(_, positions) => positions.check_children(scope).place(1),
            Variant::Enum(_, variants) => {
                let mut names = variants.names_in(0);
                for declared in &mut names {
                    declared.path.remove(0);
                    declared.path.insert(0, 1);
                }
                names.distinct()?;
                variants.check_children(scope).place(1)
            }
        }
    }
}

impl Checkable for KindDeclaration {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        self.identity.check(scope).place(0)?;
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
                let mut names = capabilities.names_in(0);
                for declared in &mut names {
                    declared.path.remove(0);
                    declared.path.insert(0, 1);
                }
                names.distinct()?;
                capabilities.check_children(&inner).place(1)
            }
            KindBody::Complex {
                superkinds,
                types,
                constants,
                capabilities,
            } => {
                for (index, superkind) in superkinds.iter().enumerate() {
                    if superkind.reaches_superkind(
                        &self.identity.name,
                        scope.file,
                        &mut vec![self.identity.name.clone()],
                    ) {
                        return Err(Fault::Conceptual(
                            vec![1, 0, index as Integer],
                            Problem::Cycle(
                                protos::Text::try_from(self.identity.name.0.clone())
                                    .expect("identifier"),
                            ),
                        ));
                    }
                }
                let mut names = types.names_in(1);
                names.extend(constants.names_in(2));
                names.extend(capabilities.names_in(3));
                for declared in &mut names {
                    declared.path.insert(0, 1);
                }
                names.distinct()?;
                superkinds.refer_each(&inner, Role::Kind, 0).place(1)?;
                types.check_each(&inner, 1).place(1)?;
                constants.check_each(&inner, 2).place(1)?;
                capabilities.check_each(&inner, 3).place(1)
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
        self.ty.check(scope).place(1)
    }
}

impl Checkable for Capability {
    fn check(&self, scope: &Scope) -> Result<(), Fault> {
        match &self.signature {
            Signature::Yielding(yields) => yields.check(scope).place(0).place(1),
            Signature::Taking(inputs, yields) => {
                inputs.check_each(scope, 0).place(0).place(1)?;
                yields.check(scope).place(0).place(1).place(1)
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
        self.identity.check(scope).place(0)?;
        // The kinds borne are named outside the identity that bears them.
        self.kinds.refer_each(scope, Role::Kind, 0).place(1)
    }
}
