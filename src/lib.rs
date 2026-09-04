use std::collections::HashMap;
use std::fmt;

use protos::{Enclosure, Extent, Head, Protoform, Separator, Structural};
use quote::{ToTokens, format_ident, quote};

// ============================================================================
// Import resolution
// ============================================================================

/// Build a resolution table from imports: name -> source module.
fn build_import_resolution(imports: &[Import]) -> HashMap<String, String> {
    let mut resolution = HashMap::new();
    for import in imports {
        match import {
            Import::Single { source, name } => {
                resolution.insert(name.clone(), source.clone());
            }
            Import::Multiple { source, names } => {
                for name in names {
                    resolution.insert(name.clone(), source.clone());
                }
            }
        }
    }
    resolution
}

// ============================================================================
// Concept types
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version(pub i64, pub i64, pub i64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Import {
    Single { source: String, name: String },
    Multiple { source: String, names: Vec<String> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Named(String),
    Applied {
        constructor: String,
        arguments: Vec<TypeExpression>,
    },
    SelfType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Unit(String),
    Typed(String, TypeExpression),
    InlineStruct(String, Vec<TypeExpression>),
    InlineEnum(String, Vec<Variant>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeDeclaration {
    Struct {
        name: String,
        fields: Vec<TypeExpression>,
    },
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
    Alias {
        name: String,
        target: TypeExpression,
    },
    Map {
        name: String,
        key: TypeExpression,
        value: TypeExpression,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Receiver {
    Shared,
    Mutable,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub name: String,
    pub receiver: Receiver,
    pub inputs: Vec<TypeExpression>,
    pub yield_type: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedType {
    pub name: String,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedConstant {
    pub name: String,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindConstraint {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KindDeclaration {
    Simple {
        name: String,
        constraints: Vec<KindConstraint>,
        capabilities: Vec<Capability>,
    },
    Complex {
        name: String,
        constraints: Vec<KindConstraint>,
        superkinds: Vec<String>,
        associated_types: Vec<AssociatedType>,
        associated_constants: Vec<AssociatedConstant>,
        capabilities: Vec<Capability>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Association {
    pub ty: String,
    pub kinds: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionReference {
    pub name: String,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Library {
    pub version: Version,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDeclaration>,
    pub kinds: Vec<KindDeclaration>,
    pub associations: Vec<Association>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signal {
    pub version: Version,
    pub imports: Vec<Import>,
    pub requests: Vec<SectionReference>,
    pub responses: Vec<SectionReference>,
    pub types: Vec<TypeDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Concept {
    Library(Library),
    Signal(Signal),
}

// ============================================================================
// Kinds and layers
// ============================================================================

/// Text that may be an ethos file, ready for actualization.
pub struct Potential(String);

impl From<&str> for Potential {
    fn from(source: &str) -> Self {
        Self(source.to_owned())
    }
}

impl From<String> for Potential {
    fn from(source: String) -> Self {
        Self(source)
    }
}

impl Potential {
    /// The source text.
    pub fn text(&self) -> &str {
        &self.0
    }
}

/// The reading kind: text actualizes into a Concept.
pub trait Actualizing {
    fn actualize(&self) -> Result<Concept, Fault>;
}

/// The emitting kind: a Concept emits generated Rust.
pub trait Emitting {
    fn emit(&self) -> Result<String, Fault>;
}

/// The generated Rust from an ethos concept.
pub struct RustLibrary(pub String);

// ---------------------------------------------------------------------------
// Layer impls: Conceptual<Concept> for Datom, Protosizable for Concept
// ---------------------------------------------------------------------------

/// Ethos is read as datom first: a Datom conceives into a Concept.
impl protos::Conceptual<Concept> for datomic::Datom {
    type Fault = Fault;

    fn conceive(&self) -> Result<Concept, Fault> {
        // The ethos reading from a Datom is the same logic as reading from
        // protoforms: protosize the datom back to a protoform and read from
        // that. This preserves the structural reading logic while honoring the
        // layer design (ethos is read as datom first).
        use protos::Protosizable;
        let protoform = self.protosize();
        read_protoform_as_concept(&protoform)
    }
}

/// A Concept yields its protoform in the sweet form (the reverse of the reader).
impl protos::Protosizable for Concept {
    fn protosize(&self) -> Protoform {
        match self {
            Concept::Library(lib) => protosize_library(lib),
            Concept::Signal(sig) => protosize_signal(sig),
        }
    }
}

fn protosize_library(lib: &Library) -> Protoform {
    // Sweet form: Library.{ver} [imports] [types] [kinds] [associations]
    // The sweet form is a single Headed followed by sibling sections.
    // But protosize yields one Protoform. So we use the full form:
    // Library.{ {ver} [imports] [types] [kinds] [associations] }
    Protoform::Headed(
        Head::Bare("Library".to_owned()),
        Separator::Period,
        Box::new(Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_version(&lib.version),
                protosize_imports(&lib.imports),
                protosize_types(&lib.types),
                protosize_kinds(&lib.kinds),
                protosize_associations(&lib.associations),
            ],
        )),
    )
}

fn protosize_signal(sig: &Signal) -> Protoform {
    Protoform::Headed(
        Head::Bare("Signal".to_owned()),
        Separator::Period,
        Box::new(Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_version(&sig.version),
                protosize_imports(&sig.imports),
                protosize_section_references(&sig.requests),
                protosize_section_references(&sig.responses),
                protosize_types(&sig.types),
            ],
        )),
    )
}

fn protosize_version(v: &Version) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Braced,
        vec![
            Protoform::Bare(v.0.to_string()),
            Protoform::Bare(v.1.to_string()),
            Protoform::Bare(v.2.to_string()),
        ],
    )
}

fn protosize_imports(imports: &[Import]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        imports.iter().map(protosize_import).collect(),
    )
}

fn protosize_import(import: &Import) -> Protoform {
    match import {
        Import::Single { source, name } => Protoform::Headed(
            Head::Bare(source.clone()),
            Separator::Colon,
            Box::new(Protoform::Bare(name.clone())),
        ),
        Import::Multiple { source, names } => Protoform::Headed(
            Head::Bare(source.clone()),
            Separator::Colon,
            Box::new(Protoform::Enclosed(
                Enclosure::Bracketed,
                names.iter().map(|n| Protoform::Bare(n.clone())).collect(),
            )),
        ),
    }
}

fn protosize_types(types: &[TypeDeclaration]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        types.iter().map(protosize_type_declaration).collect(),
    )
}

fn protosize_type_declaration(decl: &TypeDeclaration) -> Protoform {
    match decl {
        TypeDeclaration::Struct { name, fields } => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(protosize_type_expression).collect(),
            )),
        ),
        TypeDeclaration::Enum { name, variants } => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Bracketed,
                variants.iter().map(protosize_variant).collect(),
            )),
        ),
        TypeDeclaration::Alias { name, target } => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(protosize_type_expression(target)),
        ),
        TypeDeclaration::Map { name, key, value } => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Guillemets,
                vec![
                    protosize_type_expression(key),
                    protosize_type_expression(value),
                ],
            )),
        ),
    }
}

fn protosize_variant(variant: &Variant) -> Protoform {
    match variant {
        Variant::Unit(name) => Protoform::Bare(name.clone()),
        Variant::Typed(name, ty) => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(protosize_type_expression(ty)),
        ),
        Variant::InlineStruct(name, fields) => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(protosize_type_expression).collect(),
            )),
        ),
        Variant::InlineEnum(name, variants) => Protoform::Headed(
            Head::Bare(name.clone()),
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Bracketed,
                variants.iter().map(protosize_variant).collect(),
            )),
        ),
    }
}

fn protosize_type_expression(expr: &TypeExpression) -> Protoform {
    match expr {
        TypeExpression::Named(name) => Protoform::Bare(name.clone()),
        TypeExpression::Applied {
            constructor,
            arguments,
        } => Protoform::Qualified(
            constructor.clone(),
            arguments.iter().map(protosize_type_expression).collect(),
        ),
        TypeExpression::SelfType => Protoform::Bare("Self".to_owned()),
    }
}

fn protosize_kinds(kinds: &[KindDeclaration]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        kinds.iter().map(protosize_kind_declaration).collect(),
    )
}

fn protosize_kind_declaration(kind: &KindDeclaration) -> Protoform {
    let (name, constraints) = match kind {
        KindDeclaration::Simple {
            name, constraints, ..
        }
        | KindDeclaration::Complex {
            name, constraints, ..
        } => (name, constraints),
    };

    let head = if constraints.is_empty() {
        Head::Bare(name.clone())
    } else {
        Head::Qualified(
            name.clone(),
            constraints.iter().map(protosize_kind_constraint).collect(),
        )
    };

    match kind {
        KindDeclaration::Simple { capabilities, .. } => Protoform::Headed(
            head,
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Bracketed,
                capabilities.iter().map(protosize_capability).collect(),
            )),
        ),
        KindDeclaration::Complex {
            superkinds,
            associated_types,
            associated_constants,
            capabilities,
            ..
        } => Protoform::Headed(
            head,
            Separator::Period,
            Box::new(Protoform::Enclosed(
                Enclosure::Braced,
                vec![
                    Protoform::Enclosed(
                        Enclosure::Bracketed,
                        superkinds
                            .iter()
                            .map(|s| Protoform::Bare(s.clone()))
                            .collect(),
                    ),
                    Protoform::Enclosed(
                        Enclosure::Bracketed,
                        associated_types
                            .iter()
                            .map(protosize_associated_type)
                            .collect(),
                    ),
                    Protoform::Enclosed(
                        Enclosure::Guillemets,
                        associated_constants
                            .iter()
                            .flat_map(|ac| {
                                vec![
                                    Protoform::Bare(ac.name.clone()),
                                    protosize_type_expression(&ac.ty),
                                ]
                            })
                            .collect(),
                    ),
                    Protoform::Enclosed(
                        Enclosure::Bracketed,
                        capabilities.iter().map(protosize_capability).collect(),
                    ),
                ],
            )),
        ),
    }
}

fn protosize_kind_constraint(kc: &KindConstraint) -> Protoform {
    if kc.bounds.len() == 1 {
        Protoform::Bare(kc.bounds[0].clone())
    } else {
        Protoform::Enclosed(
            Enclosure::Bracketed,
            kc.bounds
                .iter()
                .map(|b| Protoform::Bare(b.clone()))
                .collect(),
        )
    }
}

fn protosize_associated_type(at: &AssociatedType) -> Protoform {
    if at.constraints.is_empty() {
        Protoform::Bare(at.name.clone())
    } else {
        Protoform::Qualified(
            at.name.clone(),
            at.constraints
                .iter()
                .map(|c| Protoform::Bare(c.clone()))
                .collect(),
        )
    }
}

fn protosize_capability(cap: &Capability) -> Protoform {
    let sep = match cap.receiver {
        Receiver::Shared => Separator::Period,
        Receiver::Mutable => Separator::Exclamation,
        Receiver::None => Separator::Colon,
    };

    let body = if cap.inputs.is_empty() {
        Protoform::Enclosed(
            Enclosure::Bracketed,
            vec![protosize_type_expression(&cap.yield_type)],
        )
    } else {
        Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                Protoform::Enclosed(
                    Enclosure::Bracketed,
                    cap.inputs.iter().map(protosize_type_expression).collect(),
                ),
                Protoform::Enclosed(
                    Enclosure::Bracketed,
                    vec![protosize_type_expression(&cap.yield_type)],
                ),
            ],
        )
    };

    Protoform::Headed(Head::Bare(cap.name.clone()), sep, Box::new(body))
}

fn protosize_associations(associations: &[Association]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        associations.iter().map(protosize_association).collect(),
    )
}

fn protosize_association(assoc: &Association) -> Protoform {
    Protoform::Headed(
        Head::Bare(assoc.ty.clone()),
        Separator::Period,
        Box::new(Protoform::Enclosed(
            Enclosure::Bracketed,
            assoc
                .kinds
                .iter()
                .map(|k| Protoform::Bare(k.clone()))
                .collect(),
        )),
    )
}

fn protosize_section_references(refs: &[SectionReference]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        refs.iter().map(protosize_section_reference).collect(),
    )
}

fn protosize_section_reference(sr: &SectionReference) -> Protoform {
    Protoform::Headed(
        Head::Bare(sr.name.clone()),
        Separator::Period,
        Box::new(protosize_type_expression(&sr.ty)),
    )
}

// ============================================================================
// Faults
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fault {
    pub extent: Extent,
    pub problem: Problem,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Problem {
    Protos,
    Root,
    Version,
    Section,
    Import,
    Declaration,
    TypeExpression,
    Capability,
    Kind,
    Association,
    Emission,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} at {}..{}",
            self.problem, self.extent.0, self.extent.1
        )
    }
}

impl std::error::Error for Fault {}

// ============================================================================
// Reader (Actualizing for Potential)
// ============================================================================

impl Actualizing for Potential {
    fn actualize(&self) -> Result<Concept, Fault> {
        let delineation = self.0.delineate().map_err(|f| Fault {
            extent: f.extent,
            problem: Problem::Protos,
        })?;
        let pfs = &delineation.protoforms;

        let first = pfs.first().ok_or_else(|| root_fault(self.0.len()))?;

        let (head, sep, body) = pf_headed(first).ok_or_else(|| fault_here(Problem::Root))?;
        if sep != Separator::Period {
            return Err(fault_here(Problem::Root));
        }

        match head {
            "Library" => read_library(body, &pfs[1..]),
            "Signal" => read_signal(body, &pfs[1..]),
            _ => Err(fault_here(Problem::Root)),
        }
    }
}

/// Read a single protoform as an ethos concept (for the Conceptual<Concept> impl).
fn read_protoform_as_concept(pf: &Protoform) -> Result<Concept, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Root))?;
    if sep != Separator::Period {
        return Err(fault_here(Problem::Root));
    }
    match head {
        "Library" => read_library(body, &[]),
        "Signal" => read_signal(body, &[]),
        _ => Err(fault_here(Problem::Root)),
    }
}

fn read_library(version_or_body: &Protoform, rest: &[Protoform]) -> Result<Concept, Fault> {
    let (version, sections) = extract_version_and_sections(version_or_body, rest, 4)?;
    Ok(Concept::Library(Library {
        version,
        imports: read_imports(sections[0])?,
        types: read_types(sections[1])?,
        kinds: read_kinds(sections[2])?,
        associations: read_associations(sections[3])?,
    }))
}

fn read_signal(version_or_body: &Protoform, rest: &[Protoform]) -> Result<Concept, Fault> {
    let (version, sections) = extract_version_and_sections(version_or_body, rest, 4)?;
    Ok(Concept::Signal(Signal {
        version,
        imports: read_imports(sections[0])?,
        requests: read_section_references(sections[1])?,
        responses: read_section_references(sections[2])?,
        types: read_types(sections[3])?,
    }))
}

fn extract_version_and_sections<'a>(
    body: &'a Protoform,
    rest: &'a [Protoform],
    expected: usize,
) -> Result<(Version, Vec<&'a Protoform>), Fault> {
    let braced = pf_braced(body).ok_or_else(|| fault_here(Problem::Version))?;

    if braced.iter().all(|p| pf_bare(p).is_some()) {
        let version = read_version(braced)?;
        if rest.len() != expected {
            return Err(fault_here(Problem::Section));
        }
        Ok((version, rest.iter().collect()))
    } else {
        if braced.len() != 1 + expected {
            return Err(fault_here(Problem::Section));
        }
        let version_children = pf_braced(&braced[0]).ok_or_else(|| fault_here(Problem::Version))?;
        let version = read_version(version_children)?;
        Ok((version, braced[1..].iter().collect()))
    }
}

fn read_version(pfs: &[Protoform]) -> Result<Version, Fault> {
    let [major, minor, patch] = pfs else {
        return Err(fault_here(Problem::Version));
    };
    Ok(Version(
        bare_integer(major)?,
        bare_integer(minor)?,
        bare_integer(patch)?,
    ))
}

fn read_imports(pf: &Protoform) -> Result<Vec<Import>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Import))?;
    children.iter().map(read_import).collect()
}

fn read_import(pf: &Protoform) -> Result<Import, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Import))?;
    if sep != Separator::Colon {
        return Err(fault_here(Problem::Import));
    }
    let source = head.to_owned();
    if let Some(names) = pf_bracketed(body) {
        let names = names
            .iter()
            .map(|p| {
                bare_symbol(p)
                    .map(str::to_owned)
                    .map_err(|()| fault_here(Problem::Import))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Import::Multiple { source, names })
    } else {
        let name = bare_symbol(body)
            .map_err(|()| fault_here(Problem::Import))?
            .to_owned();
        Ok(Import::Single { source, name })
    }
}

fn read_types(pf: &Protoform) -> Result<Vec<TypeDeclaration>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Declaration))?;
    let mut declarations = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let (decl, consumed) = read_type_declaration(&children[index], children.get(index + 1))?;
        declarations.push(decl);
        index += 1 + usize::from(consumed);
    }
    Ok(declarations)
}

fn read_type_declaration(
    pf: &Protoform,
    following: Option<&Protoform>,
) -> Result<(TypeDeclaration, bool), Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Declaration))?;
    if sep != Separator::Period {
        return Err(fault_here(Problem::Declaration));
    }
    let name = head.to_owned();

    if let Some(children) = pf_braced(body) {
        let fields = read_type_expression_list(children)?;
        return Ok((TypeDeclaration::Struct { name, fields }, false));
    }

    if let Some(children) = pf_bracketed(body) {
        let variants = read_variants(children)?;
        return Ok((TypeDeclaration::Enum { name, variants }, false));
    }

    if let Some(children) = pf_guillemets(body) {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < children.len() {
            let (expr, ate) =
                read_type_expression_with_following(&children[i], children.get(i + 1))?;
            exprs.push(expr);
            i += 1 + usize::from(ate);
        }
        if exprs.len() != 2 {
            return Err(fault_here(Problem::Declaration));
        }
        let value = exprs.pop().unwrap();
        let key = exprs.pop().unwrap();
        return Ok((TypeDeclaration::Map { name, key, value }, false));
    }

    let (target, consumed) = read_type_expression_with_following(body, following)?;
    Ok((TypeDeclaration::Alias { name, target }, consumed))
}

fn read_variants(pfs: &[Protoform]) -> Result<Vec<Variant>, Fault> {
    let mut variants = Vec::new();
    let mut index = 0;
    while index < pfs.len() {
        let (variant, consumed) = read_variant(&pfs[index], pfs.get(index + 1))?;
        variants.push(variant);
        index += 1 + usize::from(consumed);
    }
    Ok(variants)
}

fn read_variant(pf: &Protoform, following: Option<&Protoform>) -> Result<(Variant, bool), Fault> {
    if let Some((head, sep, body)) = pf_headed(pf) {
        if sep != Separator::Period {
            return Err(fault_here(Problem::Declaration));
        }
        let name = head.to_owned();

        if let Some(children) = pf_braced(body) {
            let fields = read_type_expression_list(children)?;
            return Ok((Variant::InlineStruct(name, fields), false));
        }

        if let Some(children) = pf_bracketed(body) {
            let inner = read_variants(children)?;
            return Ok((Variant::InlineEnum(name, inner), false));
        }

        let (ty, consumed) = read_type_expression_with_following(body, following)?;
        return Ok((Variant::Typed(name, ty), consumed));
    }

    let name = bare_symbol(pf)
        .map_err(|()| fault_here(Problem::Declaration))?
        .to_owned();
    Ok((Variant::Unit(name), false))
}

fn read_type_expression_list(pfs: &[Protoform]) -> Result<Vec<TypeExpression>, Fault> {
    let mut expressions = Vec::new();
    let mut index = 0;
    while index < pfs.len() {
        let (expr, consumed) =
            read_type_expression_with_following(&pfs[index], pfs.get(index + 1))?;
        expressions.push(expr);
        index += 1 + usize::from(consumed);
    }
    Ok(expressions)
}

fn read_type_expression_with_following(
    pf: &Protoform,
    following: Option<&Protoform>,
) -> Result<(TypeExpression, bool), Fault> {
    if let Protoform::Qualified(constructor, args) = pf {
        let arguments = read_type_expression_list(args)?;
        return Ok((
            TypeExpression::Applied {
                constructor: constructor.to_owned(),
                arguments,
            },
            false,
        ));
    }

    if let Ok(name) = bare_symbol(pf) {
        if name == "Self" {
            return Ok((TypeExpression::SelfType, false));
        }
        if let Some(angled) = following.and_then(pf_angled) {
            let arguments = read_type_expression_list(angled)?;
            return Ok((
                TypeExpression::Applied {
                    constructor: name.to_owned(),
                    arguments,
                },
                true,
            ));
        }
        return Ok((TypeExpression::Named(name.to_owned()), false));
    }

    Err(fault_here(Problem::TypeExpression))
}

fn read_kinds(pf: &Protoform) -> Result<Vec<KindDeclaration>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Kind))?;
    children.iter().map(read_kind).collect()
}

fn read_kind(pf: &Protoform) -> Result<KindDeclaration, Fault> {
    let (head, _sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Kind))?;
    let name = head.to_owned();

    let head_constraints = match pf_head_qualifiers(pf) {
        Some(quals) => read_kind_constraints(quals)?,
        None => Vec::new(),
    };

    if let Some(children) = pf_bracketed(body) {
        let capabilities = read_capabilities(children)?;
        return Ok(KindDeclaration::Simple {
            name,
            constraints: head_constraints,
            capabilities,
        });
    }

    if let Some(children) = pf_braced(body) {
        let (constraints, body_start) = if !head_constraints.is_empty() {
            (head_constraints, 0)
        } else if let Some(angled) = children.first().and_then(pf_angled) {
            (read_kind_constraints(angled)?, 1)
        } else {
            (Vec::new(), 0)
        };

        let body_children = &children[body_start..];

        if body_children.len() == 4 {
            let superkinds = read_bare_list(&body_children[0])?;
            let associated_types = read_associated_types(&body_children[1])?;
            let associated_constants = read_associated_constants(&body_children[2])?;
            let cap_children =
                pf_bracketed(&body_children[3]).ok_or_else(|| fault_here(Problem::Kind))?;
            let capabilities = read_capabilities(cap_children)?;
            return Ok(KindDeclaration::Complex {
                name,
                constraints,
                superkinds,
                associated_types,
                associated_constants,
                capabilities,
            });
        }

        if body_children.len() == 1
            && let Some(cap_children) = pf_bracketed(&body_children[0])
        {
            let capabilities = read_capabilities(cap_children)?;
            return Ok(KindDeclaration::Simple {
                name,
                constraints,
                capabilities,
            });
        }

        return Err(fault_here(Problem::Kind));
    }

    Err(fault_here(Problem::Kind))
}

fn read_kind_constraints(pfs: &[Protoform]) -> Result<Vec<KindConstraint>, Fault> {
    let mut constraints = Vec::new();
    for pf in pfs {
        if let Some(children) = pf_bracketed(pf) {
            let bounds = children
                .iter()
                .map(|p| {
                    bare_symbol(p)
                        .map(str::to_owned)
                        .map_err(|()| fault_here(Problem::Kind))
                })
                .collect::<Result<Vec<_>, _>>()?;
            constraints.push(KindConstraint {
                name: String::new(),
                bounds,
            });
        } else {
            let bound = bare_symbol(pf)
                .map_err(|()| fault_here(Problem::Kind))?
                .to_owned();
            constraints.push(KindConstraint {
                name: String::new(),
                bounds: vec![bound],
            });
        }
    }
    for (i, constraint) in constraints.iter_mut().enumerate() {
        constraint.name = String::from((b'A' + i as u8) as char);
    }
    Ok(constraints)
}

fn read_bare_list(pf: &Protoform) -> Result<Vec<String>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Kind))?;
    children
        .iter()
        .map(|p| {
            bare_symbol(p)
                .map(str::to_owned)
                .map_err(|()| fault_here(Problem::Kind))
        })
        .collect()
}

fn read_associated_types(pf: &Protoform) -> Result<Vec<AssociatedType>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Kind))?;
    let mut types = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let child = &children[index];
        let name = bare_symbol(child)
            .map_err(|()| fault_here(Problem::Kind))?
            .to_owned();
        if let Some(angled) = children.get(index + 1).and_then(pf_angled) {
            let constraints = angled
                .iter()
                .map(|p| {
                    bare_symbol(p)
                        .map(str::to_owned)
                        .map_err(|()| fault_here(Problem::Kind))
                })
                .collect::<Result<Vec<_>, _>>()?;
            types.push(AssociatedType { name, constraints });
            index += 2;
        } else {
            types.push(AssociatedType {
                name,
                constraints: Vec::new(),
            });
            index += 1;
        }
    }
    Ok(types)
}

fn read_associated_constants(pf: &Protoform) -> Result<Vec<AssociatedConstant>, Fault> {
    let children = pf_guillemets(pf).ok_or_else(|| fault_here(Problem::Kind))?;
    let mut constants = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let name = bare_symbol(&children[index])
            .map_err(|()| fault_here(Problem::Kind))?
            .to_owned();
        index += 1;
        if index >= children.len() {
            return Err(fault_here(Problem::Kind));
        }
        let (ty, consumed) =
            read_type_expression_with_following(&children[index], children.get(index + 1))?;
        constants.push(AssociatedConstant { name, ty });
        index += 1 + usize::from(consumed);
    }
    Ok(constants)
}

fn read_capabilities(pfs: &[Protoform]) -> Result<Vec<Capability>, Fault> {
    pfs.iter().map(read_capability).collect()
}

fn read_capability(pf: &Protoform) -> Result<Capability, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Capability))?;
    let name = head.to_owned();
    let receiver = match sep {
        Separator::Period => Receiver::Shared,
        Separator::Exclamation => Receiver::Mutable,
        Separator::Colon => Receiver::None,
    };

    if let Some(children) = pf_bracketed(body) {
        let yield_type = read_single_type_expression(children)?;
        return Ok(Capability {
            name,
            receiver,
            inputs: Vec::new(),
            yield_type,
        });
    }

    if let Some(children) = pf_braced(body) {
        if children.len() != 2 {
            return Err(fault_here(Problem::Capability));
        }
        let input_children =
            pf_bracketed(&children[0]).ok_or_else(|| fault_here(Problem::Capability))?;
        let inputs = read_type_expression_list(input_children)?;
        let yield_children =
            pf_bracketed(&children[1]).ok_or_else(|| fault_here(Problem::Capability))?;
        let yield_type = read_single_type_expression(yield_children)?;
        return Ok(Capability {
            name,
            receiver,
            inputs,
            yield_type,
        });
    }

    Err(fault_here(Problem::Capability))
}

fn read_single_type_expression(pfs: &[Protoform]) -> Result<TypeExpression, Fault> {
    if pfs.is_empty() {
        return Err(emit_fault());
    }
    let (expr, consumed) = read_type_expression_with_following(&pfs[0], pfs.get(1))?;
    let expected_len = 1 + usize::from(consumed);
    if pfs.len() != expected_len {
        return Err(fault_here(Problem::TypeExpression));
    }
    Ok(expr)
}

fn read_associations(pf: &Protoform) -> Result<Vec<Association>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Association))?;
    children.iter().map(read_association).collect()
}

fn read_association(pf: &Protoform) -> Result<Association, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Association))?;
    if sep != Separator::Period {
        return Err(fault_here(Problem::Association));
    }
    let ty = head.to_owned();
    let kinds_children = pf_bracketed(body).ok_or_else(|| fault_here(Problem::Association))?;
    let kinds = kinds_children
        .iter()
        .map(|p| {
            bare_symbol(p)
                .map(str::to_owned)
                .map_err(|()| fault_here(Problem::Association))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Association { ty, kinds })
}

fn read_section_references(pf: &Protoform) -> Result<Vec<SectionReference>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault_here(Problem::Section))?;
    let mut refs = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let (reference, consumed) =
            read_section_reference(&children[index], children.get(index + 1))?;
        refs.push(reference);
        index += 1 + usize::from(consumed);
    }
    Ok(refs)
}

fn read_section_reference(
    pf: &Protoform,
    following: Option<&Protoform>,
) -> Result<(SectionReference, bool), Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault_here(Problem::Section))?;
    if sep != Separator::Period {
        return Err(fault_here(Problem::Section));
    }
    let name = head.to_owned();
    let (ty, consumed) = read_type_expression_with_following(body, following)?;
    Ok((SectionReference { name, ty }, consumed))
}

// ============================================================================
// Emitter (Emitting for Concept)
// ============================================================================

impl Emitting for Concept {
    fn emit(&self) -> Result<String, Fault> {
        let tokens = emit_tokens(self)?;
        let syntax: syn::File = syn::parse2(tokens).map_err(|_| emit_fault())?;
        Ok(syntax.into_token_stream().to_string())
    }
}

fn emit_tokens(concept: &Concept) -> Result<proc_macro2::TokenStream, Fault> {
    let mut tokens = quote! { #![allow(dead_code)] };

    let imports = match concept {
        Concept::Library(lib) => build_import_resolution(&lib.imports),
        Concept::Signal(sig) => build_import_resolution(&sig.imports),
    };

    match concept {
        Concept::Library(library) => {
            for ty in &library.types {
                tokens.extend(type_declaration_tokens(ty, false, &imports)?);
                tokens.extend(datomic_impl_tokens(ty, &imports)?);
            }
            for kind in &library.kinds {
                tokens.extend(kind_declaration_tokens(kind, &imports)?);
            }
            for assoc in &library.associations {
                tokens.extend(association_assertion_tokens(assoc)?);
            }
        }
        Concept::Signal(signal) => {
            tokens.extend(quote! {
                use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
            });
            for ty in &signal.types {
                tokens.extend(type_declaration_tokens(ty, true, &imports)?);
                tokens.extend(datomic_impl_tokens(ty, &imports)?);
            }
            tokens.extend(section_enum_tokens(
                "Request",
                &signal.requests,
                true,
                &imports,
            )?);
            tokens.extend(section_enum_tokens(
                "Reply",
                &signal.responses,
                true,
                &imports,
            )?);
            tokens.extend(wire_envelope_tokens(signal)?);
        }
    }

    Ok(tokens)
}

fn all_variants_unit(variants: &[Variant]) -> bool {
    !variants.is_empty() && variants.iter().all(|v| matches!(v, Variant::Unit(_)))
}

fn enum_derive(signal: bool, unit_only: bool) -> proc_macro2::TokenStream {
    match (signal, unit_only) {
        (true, true) => {
            quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)] }
        }
        (true, false) => {
            quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
        }
        (false, true) => {
            quote! { #[derive(Clone, Copy, Debug, PartialEq, Eq)] }
        }
        (false, false) => {
            quote! { #[derive(Clone, Debug, PartialEq, Eq)] }
        }
    }
}

fn type_declaration_tokens(
    decl: &TypeDeclaration,
    signal: bool,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    let derive = if signal {
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
    } else {
        quote! { #[derive(Clone, Debug, PartialEq, Eq)] }
    };

    Ok(match decl {
        TypeDeclaration::Struct { name, fields } => {
            let name = ident(name)?;
            let field_tokens = fields
                .iter()
                .map(|ty| {
                    let ty = type_expression_tokens(ty, imports)?;
                    Ok(quote! { pub #ty })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            quote! { #derive pub struct #name ( #( #field_tokens, )* ); }
        }
        TypeDeclaration::Enum { name, variants } => {
            let name_ident = ident(name)?;
            // Detect self-referential variants: a position that directly names
            // this enum requires Box indirection.
            let recursive = variants_have_recursive_ref(variants, name);
            let box_ctx = recursive.then_some(name.as_str());
            let (variant_tokens, inline_types) =
                emit_variant_tokens(&name_ident, variants, signal, imports, box_ctx)?;
            // When recursive, emit impl_datomic_box! so Box<Name> is Datomic.
            let box_impl = if recursive {
                quote! { datomic::impl_datomic_box!(#name_ident); }
            } else {
                proc_macro2::TokenStream::new()
            };
            let derive = enum_derive(signal, all_variants_unit(variants));
            quote! {
                #( #inline_types )*
                #derive pub enum #name_ident { #( #variant_tokens, )* }
                #box_impl
            }
        }
        TypeDeclaration::Alias { name, target } => {
            // Always a type alias, even in Signal roots. An alias of an
            // rkyv-able type needs no derive; the underlying type already has
            // the Corporal/Datomic impls.
            let name = ident(name)?;
            let target = type_expression_tokens(target, imports)?;
            quote! { pub type #name = #target; }
        }
        TypeDeclaration::Map { name, key, value } => {
            let name = ident(name)?;
            let key = type_expression_tokens(key, imports)?;
            let value = type_expression_tokens(value, imports)?;
            quote! { pub type #name = std::collections::BTreeMap<#key, #value>; }
        }
    })
}

fn emit_variant_tokens(
    parent: &proc_macro2::Ident,
    variants: &[Variant],
    signal: bool,
    imports: &HashMap<String, String>,
    // When Some(name), fields directly naming `name` are wrapped in Box.
    box_name: Option<&str>,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), Fault> {
    let derive = if signal {
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
    } else {
        quote! { #[derive(Clone, Debug, PartialEq, Eq)] }
    };

    let mut variant_tokens = Vec::new();
    let mut inline_types = Vec::new();

    for variant in variants {
        match variant {
            Variant::Unit(name) => {
                let name = ident(name)?;
                variant_tokens.push(quote! { #name });
            }
            Variant::Typed(name, ty) => {
                let name = ident(name)?;
                let ty = if let Some(bn) = box_name {
                    type_expression_tokens_boxed(ty, imports, bn)?
                } else {
                    type_expression_tokens(ty, imports)?
                };
                variant_tokens.push(quote! { #name(#ty) });
            }
            Variant::InlineStruct(name, fields) => {
                let variant_name = ident(name)?;
                let inline_name = format_ident!("{}{}", parent, variant_name);
                let field_tokens = fields
                    .iter()
                    .map(|ty| {
                        let ty = if let Some(bn) = box_name {
                            type_expression_tokens_boxed(ty, imports, bn)?
                        } else {
                            type_expression_tokens(ty, imports)?
                        };
                        Ok(quote! { pub #ty })
                    })
                    .collect::<Result<Vec<_>, Fault>>()?;
                inline_types
                    .push(quote! { #derive pub struct #inline_name ( #( #field_tokens, )* ); });
                variant_tokens.push(quote! { #variant_name(#inline_name) });
            }
            Variant::InlineEnum(name, inner_variants) => {
                let variant_name = ident(name)?;
                let inline_name = format_ident!("{}{}", parent, variant_name);
                let (inner_variant_tokens, inner_inline_types) =
                    emit_variant_tokens(&inline_name, inner_variants, signal, imports, box_name)?;
                inline_types.extend(inner_inline_types);
                let inline_derive = enum_derive(signal, all_variants_unit(inner_variants));
                inline_types.push(
                    quote! { #inline_derive pub enum #inline_name { #( #inner_variant_tokens, )* } },
                );
                variant_tokens.push(quote! { #variant_name(#inline_name) });
            }
        }
    }

    Ok((variant_tokens, inline_types))
}

fn type_expression_tokens(
    expr: &TypeExpression,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match expr {
        TypeExpression::Named(name) => match name.as_str() {
            "Text" => quote! { protos::Text },
            "Integer" => quote! { protos::Integer },
            "Decimal" => quote! { protos::Decimal },
            "Boolean" => quote! { protos::Boolean },
            "Meaning" => quote! { datomic::Meaning },
            "Symbol" => quote! { protos::Symbol },
            _ => {
                if let Some(module) = imports.get(name.as_str()) {
                    let module = ident(module)?;
                    let name = ident(name)?;
                    quote! { #module :: #name }
                } else {
                    let name = ident(name)?;
                    quote! { #name }
                }
            }
        },
        TypeExpression::Applied {
            constructor,
            arguments,
        } => {
            let args = arguments
                .iter()
                .map(|a| type_expression_tokens(a, imports))
                .collect::<Result<Vec<_>, _>>()?;
            match constructor.as_str() {
                "Vector" => {
                    let [inner] = args.as_slice() else {
                        return Err(emit_fault());
                    };
                    quote! { Vec<#inner> }
                }
                "Option" => {
                    let [inner] = args.as_slice() else {
                        return Err(emit_fault());
                    };
                    quote! { Option<#inner> }
                }
                "Result" => {
                    let [ok, err] = args.as_slice() else {
                        return Err(emit_fault());
                    };
                    quote! { Result<#ok, #err> }
                }
                "Box" => {
                    let [inner] = args.as_slice() else {
                        return Err(emit_fault());
                    };
                    quote! { Box<#inner> }
                }
                _ => {
                    if let Some(module) = imports.get(constructor.as_str()) {
                        let module = ident(module)?;
                        let name = ident(constructor)?;
                        quote! { #module :: #name < #( #args ),* > }
                    } else {
                        let name = ident(constructor)?;
                        quote! { #name< #( #args ),* > }
                    }
                }
            }
        }
        TypeExpression::SelfType => quote! { Self },
    })
}

// ---------------------------------------------------------------------------
// Recursive-type helpers
// ---------------------------------------------------------------------------

/// True if `ty` directly names `enclosing` (one-step self-reference).
fn is_direct_recursive(ty: &TypeExpression, enclosing: &str) -> bool {
    matches!(ty, TypeExpression::Named(n) if n == enclosing)
}

/// True if any field in `fields` directly names `enclosing`.
fn fields_have_recursive_ref(fields: &[TypeExpression], enclosing: &str) -> bool {
    fields.iter().any(|f| is_direct_recursive(f, enclosing))
}

/// True if any variant of this enum (at any depth) has a direct recursive ref.
fn variants_have_recursive_ref(variants: &[Variant], enclosing: &str) -> bool {
    variants.iter().any(|v| match v {
        Variant::Unit(_) => false,
        Variant::Typed(_, ty) => is_direct_recursive(ty, enclosing),
        Variant::InlineStruct(_, fields) => fields_have_recursive_ref(fields, enclosing),
        Variant::InlineEnum(_, inner) => variants_have_recursive_ref(inner, enclosing),
    })
}

/// Emit `ty` as tokens, wrapping it in `Box<…>` when it directly names `box_name`.
fn type_expression_tokens_boxed(
    expr: &TypeExpression,
    imports: &HashMap<String, String>,
    box_name: &str,
) -> Result<proc_macro2::TokenStream, Fault> {
    if is_direct_recursive(expr, box_name) {
        let inner = type_expression_tokens(expr, imports)?;
        return Ok(quote! { Box<#inner> });
    }
    type_expression_tokens(expr, imports)
}

fn datomic_impl_tokens(
    decl: &TypeDeclaration,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    datomic_impl_tokens_with_boxing(decl, imports, None)
}

fn datomic_impl_tokens_with_boxing(
    decl: &TypeDeclaration,
    imports: &HashMap<String, String>,
    // When Some(name), fields/variants whose type directly names `name` are wrapped in Box.
    box_name: Option<&str>,
) -> Result<proc_macro2::TokenStream, Fault> {
    match decl {
        TypeDeclaration::Alias { .. } | TypeDeclaration::Map { .. } => {
            // Type aliases and map aliases have no separate Corporal/Datomic
            // impls; the underlying type already carries them.
            Ok(proc_macro2::TokenStream::new())
        }
        TypeDeclaration::Struct { name, fields } => {
            let name = ident(name)?;
            let arity = fields.len();
            let arity_i64 = arity as i64;
            let field_incorporates = fields
                .iter()
                .map(|ty| {
                    let ty = if let Some(bn) = box_name {
                        type_expression_tokens_boxed(ty, imports, bn)?
                    } else {
                        type_expression_tokens(ty, imports)?
                    };
                    Ok(quote! { <#ty as datomic::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())? })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            let field_datomizes = (0..fields.len()).map(|i| {
                let idx = syn::Index::from(i);
                quote! { datomic::Datomic::datomize(&self.#idx) }
            });
            Ok(quote! {
                impl datomic::Corporal<datomic::Datom> for #name {
                    type Fault = datomic::Fault;
                    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                        match concept {
                            datomic::Datom::Struct(fields) if fields.len() == #arity => {
                                let mut iter = fields.into_iter();
                                Ok(Self( #( #field_incorporates, )* ))
                            }
                            datomic::Datom::Struct(fields) => {
                                Err(datomic::Fault::Corporal(vec![], datomic::Problem::Arity(#arity_i64, fields.len() as i64)))
                            }
                            other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Struct, other))),
                        }
                    }
                }
                impl datomic::Datomic for #name {
                    fn datomize(&self) -> datomic::Datom {
                        datomic::Datom::Struct(vec![ #( #field_datomizes, )* ])
                    }
                }
            })
        }
        TypeDeclaration::Enum { name, variants } => {
            let name_ident = ident(name)?;
            // Pass boxing context: recursive refs inside this enum use Box.
            let recursive_boxing: Option<&str> = if variants_have_recursive_ref(variants, name) {
                Some(name.as_str())
            } else {
                box_name
            };
            let incorporate_arms = variants
                .iter()
                .map(|v| variant_incorporate_arm(&name_ident, v, imports, recursive_boxing))
                .collect::<Result<Vec<_>, _>>()?;
            let datomize_arms = variants
                .iter()
                .map(|v| variant_datomize_arm(&name_ident, v))
                .collect::<Result<Vec<_>, _>>()?;
            let nested = variants
                .iter()
                .filter_map(|v| {
                    nested_datomic_impl(&name_ident, name, v, imports, recursive_boxing).transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl datomic::Corporal<datomic::Datom> for #name_ident {
                    type Fault = datomic::Fault;
                    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                        match concept {
                            #( #incorporate_arms )*
                            other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                        }
                    }
                }
                impl datomic::Datomic for #name_ident {
                    fn datomize(&self) -> datomic::Datom {
                        match self {
                            #( #datomize_arms )*
                        }
                    }
                }
                #( #nested )*
            })
        }
    }
}

fn variant_incorporate_arm(
    parent: &proc_macro2::Ident,
    variant: &Variant,
    imports: &HashMap<String, String>,
    box_name: Option<&str>,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match variant {
        Variant::Unit(name) => {
            let name = ident(name)?;
            quote! {
                datomic::Datom::Bare(s) if s == stringify!(#name) => Ok(Self::#name),
            }
        }
        Variant::Typed(name, ty) => {
            let variant_name = ident(name)?;
            // When the field directly names the recursive type, we stored Box<T>;
            // use Box::new(T::incorporate(body)?) to construct it.
            let recursive = box_name.is_some_and(|bn| is_direct_recursive(ty, bn));
            let inner_ty = type_expression_tokens(ty, imports)?;
            if recursive {
                quote! {
                    datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                        Ok(Self::#variant_name(Box::new(<#inner_ty as datomic::Corporal<datomic::Datom>>::incorporate(*body)?)))
                    }
                }
            } else {
                quote! {
                    datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                        Ok(Self::#variant_name(<#inner_ty as datomic::Corporal<datomic::Datom>>::incorporate(*body)?))
                    }
                }
            }
        }
        Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = ident(name)?;
            let inline_name = format_ident!("{}{}", parent, variant_name);
            quote! {
                datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                    Ok(Self::#variant_name(<#inline_name as datomic::Corporal<datomic::Datom>>::incorporate(*body)?))
                }
            }
        }
    })
}

fn variant_datomize_arm(
    _parent: &proc_macro2::Ident,
    variant: &Variant,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match variant {
        Variant::Unit(name) => {
            let name = ident(name)?;
            quote! {
                Self::#name => datomic::Datom::Bare(stringify!(#name).to_owned()),
            }
        }
        Variant::Typed(name, _) | Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = ident(name)?;
            quote! {
                Self::#variant_name(value) => datomic::Datom::Variant(
                    stringify!(#variant_name).to_owned(),
                    protos::Separator::Period,
                    Some(Box::new(datomic::Datomic::datomize(value))),
                ),
            }
        }
    })
}

fn nested_datomic_impl(
    _parent: &proc_macro2::Ident,
    parent_name: &str,
    variant: &Variant,
    imports: &HashMap<String, String>,
    box_name: Option<&str>,
) -> Result<Option<proc_macro2::TokenStream>, Fault> {
    match variant {
        Variant::InlineStruct(vname, fields) => {
            let inline_name = format!("{}{}", parent_name, vname);
            Ok(Some(datomic_impl_tokens_with_boxing(
                &TypeDeclaration::Struct {
                    name: inline_name,
                    fields: fields.clone(),
                },
                imports,
                box_name,
            )?))
        }
        Variant::InlineEnum(vname, inner) => {
            let inline_name = format!("{}{}", parent_name, vname);
            Ok(Some(datomic_impl_tokens_with_boxing(
                &TypeDeclaration::Enum {
                    name: inline_name,
                    variants: inner.clone(),
                },
                imports,
                box_name,
            )?))
        }
        _ => Ok(None),
    }
}

fn kind_declaration_tokens(
    kind: &KindDeclaration,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    let (name, constraints, superkinds, associated_types, associated_constants, capabilities) =
        match kind {
            KindDeclaration::Simple {
                name,
                constraints,
                capabilities,
            } => (
                name,
                constraints.as_slice(),
                &[][..],
                &[][..],
                &[][..],
                capabilities.as_slice(),
            ),
            KindDeclaration::Complex {
                name,
                constraints,
                superkinds,
                associated_types,
                associated_constants,
                capabilities,
            } => (
                name,
                constraints.as_slice(),
                superkinds.as_slice(),
                associated_types.as_slice(),
                associated_constants.as_slice(),
                capabilities.as_slice(),
            ),
        };

    let name = ident(name)?;

    let generic_params = if constraints.is_empty() {
        proc_macro2::TokenStream::new()
    } else {
        let params = constraints
            .iter()
            .map(|c| {
                let cname = ident(&c.name)?;
                let bounds = c
                    .bounds
                    .iter()
                    .map(|b| ident(b))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(if bounds.is_empty() {
                    quote! { #cname }
                } else {
                    quote! { #cname: #( #bounds )+* }
                })
            })
            .collect::<Result<Vec<_>, Fault>>()?;
        quote! { < #( #params ),* > }
    };

    let supertrait_bounds = if superkinds.is_empty() {
        proc_macro2::TokenStream::new()
    } else {
        let bounds = superkinds
            .iter()
            .map(|s| ident(s))
            .collect::<Result<Vec<_>, _>>()?;
        quote! { : #( #bounds )+* }
    };

    let associated_type_tokens = associated_types
        .iter()
        .map(|at| {
            let aname = ident(&at.name)?;
            if at.constraints.is_empty() {
                Ok(quote! { type #aname; })
            } else {
                let bounds = at
                    .constraints
                    .iter()
                    .map(|b| ident(b))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(quote! { type #aname: #( #bounds )+*; })
            }
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let constant_tokens = associated_constants
        .iter()
        .map(|ac| {
            let cname = ident(&ac.name)?;
            let ty = type_expression_tokens(&ac.ty, imports)?;
            Ok(quote! { const #cname: #ty; })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let capability_tokens = capabilities
        .iter()
        .map(|cap| capability_method_tokens(cap, imports))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        pub trait #name #generic_params #supertrait_bounds {
            #( #associated_type_tokens )*
            #( #constant_tokens )*
            #( #capability_tokens )*
        }
    })
}

fn capability_method_tokens(
    cap: &Capability,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    let name = format_ident!("{}", &cap.name);
    let return_type = type_expression_tokens(&cap.yield_type, imports)?;

    let receiver = match cap.receiver {
        Receiver::Shared => quote! { &self },
        Receiver::Mutable => quote! { &mut self },
        Receiver::None => proc_macro2::TokenStream::new(),
    };

    let input_params = cap
        .inputs
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let pname = format_ident!("input_{}", i);
            let ty = type_expression_tokens(ty, imports)?;
            Ok(quote! { #pname: #ty })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let params = if receiver.is_empty() {
        quote! { #( #input_params ),* }
    } else if input_params.is_empty() {
        receiver
    } else {
        quote! { #receiver, #( #input_params ),* }
    };

    Ok(quote! { fn #name( #params ) -> #return_type; })
}

fn association_assertion_tokens(assoc: &Association) -> Result<proc_macro2::TokenStream, Fault> {
    let ty = ident(&assoc.ty)?;
    let assertions = assoc
        .kinds
        .iter()
        .map(|kind_name| {
            let assertion_fn = format_ident!(
                "assert_{}_{}",
                assoc.ty.to_lowercase(),
                kind_name.to_lowercase()
            );
            let kind = ident(kind_name)?;
            Ok(quote! {
                fn #assertion_fn<T: #kind>() {}
                let _ = #assertion_fn::<#ty>;
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    Ok(quote! {
        const _: () = { #( #assertions )* };
    })
}

fn section_enum_tokens(
    name: &str,
    references: &[SectionReference],
    signal: bool,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    if references.is_empty() {
        return Ok(proc_macro2::TokenStream::new());
    }
    let enum_name = ident(name)?;
    let derive = if signal {
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
    } else {
        proc_macro2::TokenStream::new()
    };
    let variants = references
        .iter()
        .map(|r| {
            let variant_name = ident(&r.name)?;
            let ty = type_expression_tokens(&r.ty, imports)?;
            Ok(quote! { #variant_name(#ty) })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let incorporate_arms = references
        .iter()
        .map(|r| {
            let variant_name = ident(&r.name)?;
            let ty = type_expression_tokens(&r.ty, imports)?;
            Ok(quote! {
                datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                    Ok(Self::#variant_name(<#ty as datomic::Corporal<datomic::Datom>>::incorporate(*body)?))
                }
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let datomize_arms = references
        .iter()
        .map(|r| {
            let variant_name = ident(&r.name)?;
            Ok(quote! {
                Self::#variant_name(value) => datomic::Datom::Variant(
                    stringify!(#variant_name).to_owned(),
                    protos::Separator::Period,
                    Some(Box::new(datomic::Datomic::datomize(value))),
                ),
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    Ok(quote! {
        #derive pub enum #enum_name { #( #variants, )* }
        impl datomic::Corporal<datomic::Datom> for #enum_name {
            type Fault = datomic::Fault;
            fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match concept {
                    #( #incorporate_arms )*
                    other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                }
            }
        }
        impl datomic::Datomic for #enum_name {
            fn datomize(&self) -> datomic::Datom {
                match self { #( #datomize_arms )* }
            }
        }
    })
}

fn wire_envelope_tokens(signal: &Signal) -> Result<proc_macro2::TokenStream, Fault> {
    let major = signal.version.0 as u16;
    let minor = signal.version.1 as u16;
    let patch = signal.version.2 as u16;

    Ok(quote! {
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
        pub struct Version(pub u16, pub u16, pub u16);

        pub const SIGNAL_VERSION: Version = Version(#major, #minor, #patch);

        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
        pub enum Refusal {
            VersionMismatch(Version, Version),
            Unreadable,
        }

        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
        pub enum Body {
            Request(Request),
            Reply(Reply),
            Refusal(Refusal),
        }

        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
        pub struct Frame(pub Version, pub Body);

        // Every generated type crosses the text boundary.
        // Version fields are u16 but the datom dialect knows Integer (i64);
        // incorporate reads as i64 and truncates, datomize widens.
        impl datomic::Corporal<datomic::Datom> for Version {
            type Fault = datomic::Fault;
            fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match concept {
                    datomic::Datom::Struct(fields) if fields.len() == 3 => {
                        let mut it = fields.into_iter();
                        let a = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())? as u16;
                        let b = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())? as u16;
                        let c = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())? as u16;
                        Ok(Self(a, b, c))
                    }
                    datomic::Datom::Struct(fields) => {
                        Err(datomic::Fault::Corporal(vec![], datomic::Problem::Arity(3, fields.len() as i64)))
                    }
                    other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Struct, other))),
                }
            }
        }
        impl datomic::Datomic for Version {
            fn datomize(&self) -> datomic::Datom {
                datomic::Datom::Struct(vec![
                    datomic::Datomic::datomize(&(self.0 as protos::Integer)),
                    datomic::Datomic::datomize(&(self.1 as protos::Integer)),
                    datomic::Datomic::datomize(&(self.2 as protos::Integer)),
                ])
            }
        }

        impl datomic::Corporal<datomic::Datom> for Refusal {
            type Fault = datomic::Fault;
            fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match concept {
                    datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                        if head == "VersionMismatch" =>
                    {
                        match *body {
                            datomic::Datom::Struct(fields) if fields.len() == 2 => {
                                let mut it = fields.into_iter();
                                Ok(Self::VersionMismatch(
                                    <Version as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())?,
                                    <Version as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())?,
                                ))
                            }
                            other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Struct, other))),
                        }
                    }
                    datomic::Datom::Bare(s) if s == "Unreadable" => Ok(Self::Unreadable),
                    other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                }
            }
        }
        impl datomic::Datomic for Refusal {
            fn datomize(&self) -> datomic::Datom {
                match self {
                    Self::VersionMismatch(a, b) => datomic::Datom::Variant(
                        "VersionMismatch".to_owned(),
                        protos::Separator::Period,
                        Some(Box::new(datomic::Datom::Struct(vec![
                            datomic::Datomic::datomize(a),
                            datomic::Datomic::datomize(b),
                        ]))),
                    ),
                    Self::Unreadable => datomic::Datom::Bare("Unreadable".to_owned()),
                }
            }
        }

        impl datomic::Corporal<datomic::Datom> for Body {
            type Fault = datomic::Fault;
            fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match concept {
                    datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) => {
                        match head.as_str() {
                            "Request" => Ok(Self::Request(
                                <Request as datomic::Corporal<datomic::Datom>>::incorporate(*body)?)),
                            "Reply" => Ok(Self::Reply(
                                <Reply as datomic::Corporal<datomic::Datom>>::incorporate(*body)?)),
                            "Refusal" => Ok(Self::Refusal(
                                <Refusal as datomic::Corporal<datomic::Datom>>::incorporate(*body)?)),
                            _ => Err(datomic::Fault::Corporal(vec![], datomic::Problem::UnknownVariant(head))),
                        }
                    }
                    other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                }
            }
        }
        impl datomic::Datomic for Body {
            fn datomize(&self) -> datomic::Datom {
                match self {
                    Self::Request(v) => datomic::Datom::Variant(
                        "Request".to_owned(), protos::Separator::Period,
                        Some(Box::new(datomic::Datomic::datomize(v)))),
                    Self::Reply(v) => datomic::Datom::Variant(
                        "Reply".to_owned(), protos::Separator::Period,
                        Some(Box::new(datomic::Datomic::datomize(v)))),
                    Self::Refusal(v) => datomic::Datom::Variant(
                        "Refusal".to_owned(), protos::Separator::Period,
                        Some(Box::new(datomic::Datomic::datomize(v)))),
                }
            }
        }

        impl datomic::Corporal<datomic::Datom> for Frame {
            type Fault = datomic::Fault;
            fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match concept {
                    datomic::Datom::Struct(fields) if fields.len() == 2 => {
                        let mut it = fields.into_iter();
                        Ok(Self(
                            <Version as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())?,
                            <Body as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())?,
                        ))
                    }
                    datomic::Datom::Struct(fields) => {
                        Err(datomic::Fault::Corporal(vec![], datomic::Problem::Arity(2, fields.len() as i64)))
                    }
                    other => Err(datomic::Fault::Corporal(vec![], datomic::Problem::Shape(datomic::Expected::Struct, other))),
                }
            }
        }
        impl datomic::Datomic for Frame {
            fn datomize(&self) -> datomic::Datom {
                datomic::Datom::Struct(vec![
                    datomic::Datomic::datomize(&self.0),
                    datomic::Datomic::datomize(&self.1),
                ])
            }
        }
    })
}

// ============================================================================
// Protoform helpers
// ============================================================================

fn pf_headed(pf: &Protoform) -> Option<(&str, Separator, &Protoform)> {
    match pf {
        Protoform::Headed(Head::Bare(head), sep, body) => Some((head.as_str(), *sep, body)),
        Protoform::Headed(Head::Qualified(head, _), sep, body) => Some((head.as_str(), *sep, body)),
        _ => None,
    }
}

fn pf_head_qualifiers(pf: &Protoform) -> Option<&[Protoform]> {
    match pf {
        Protoform::Headed(Head::Qualified(_, quals), _, _) => Some(quals),
        _ => None,
    }
}

fn pf_bare(pf: &Protoform) -> Option<&str> {
    match pf {
        Protoform::Bare(s) => Some(s.as_str()),
        _ => None,
    }
}

fn pf_enclosed(pf: &Protoform, enclosure: Enclosure) -> Option<&[Protoform]> {
    match pf {
        Protoform::Enclosed(enc, children) if *enc == enclosure => Some(children),
        _ => None,
    }
}

fn pf_braced(pf: &Protoform) -> Option<&[Protoform]> {
    pf_enclosed(pf, Enclosure::Braced)
}

fn pf_bracketed(pf: &Protoform) -> Option<&[Protoform]> {
    pf_enclosed(pf, Enclosure::Bracketed)
}

fn pf_guillemets(pf: &Protoform) -> Option<&[Protoform]> {
    pf_enclosed(pf, Enclosure::Guillemets)
}

fn pf_angled(pf: &Protoform) -> Option<&[Protoform]> {
    pf_enclosed(pf, Enclosure::Angled)
}

fn bare_symbol(pf: &Protoform) -> Result<&str, ()> {
    pf_bare(pf).ok_or(())
}

fn bare_integer(pf: &Protoform) -> Result<i64, Fault> {
    let s = bare_symbol(pf).map_err(|()| fault_here(Problem::Version))?;
    s.parse::<i64>().map_err(|_| fault_here(Problem::Version))
}

fn fault_here(problem: Problem) -> Fault {
    Fault {
        extent: Extent(0, 0),
        problem,
    }
}

fn root_fault(source_len: usize) -> Fault {
    Fault {
        extent: Extent(0, source_len as i64),
        problem: Problem::Root,
    }
}

fn emit_fault() -> Fault {
    Fault {
        extent: Extent(0, 0),
        problem: Problem::Emission,
    }
}

fn ident(name: &str) -> Result<proc_macro2::Ident, Fault> {
    Ok(format_ident!("{}", name))
}
