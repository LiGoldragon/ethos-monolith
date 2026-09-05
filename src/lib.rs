//! Ethos-zero: the ethos schema language, version zero.
//!
//! Reads ethos, emits Rust. The layers:
//!   Text → Protoform (protos delineation)
//!   Protoform → File (Conceivable<File>, the reader)
//!   File → Rust Text (Generating, the emitter)
//!
//! Sweet form conversion (Canonicalizing) runs on text before
//! delineation: the variant head becomes a headed braced structure.

use std::collections::HashMap;

use protos::{Delineation, Enclosure, Head, Protoform, Separator};
use quote::{ToTokens, format_ident, quote};

// ============================================================================
// Declaration model — the Concept layer
// ============================================================================

/// The unit of declaration: one file, one Rust module.
/// An enum of four variants, each a different kind of declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum File {
    Types(Types),
    Kinds(Kinds),
    Signal(Signal),
    Sema(Sema),
}

/// A types file: imports, type declarations, and associations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Types {
    pub imports: Vec<Import>,
    pub types: Vec<TypeDeclaration>,
    pub associations: Vec<Association>,
}

/// A kinds file: imports and kind declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kinds {
    pub imports: Vec<Import>,
    pub kinds: Vec<KindDeclaration>,
}

/// A signal file: imports, request variants, response variants, and types.
/// Generates `Request` and `Response` enums.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signal {
    pub imports: Vec<Import>,
    pub requests: Vec<Variant>,
    pub responses: Vec<Variant>,
    pub types: Vec<TypeDeclaration>,
}

/// A sema file: imports and storage/record types with implied Datomic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sema {
    pub imports: Vec<Import>,
    pub types: Vec<TypeDeclaration>,
}

// ---------------------------------------------------------------------------
// Shared declaration types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Import {
    Single(protos::Text, protos::Text),
    Multiple(protos::Text, Vec<protos::Text>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Named(protos::Text),
    Applied(protos::Text, Vec<TypeExpression>),
    SelfType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Unit(protos::Text),
    Typed(protos::Text, TypeExpression),
    InlineStruct(protos::Text, Vec<TypeExpression>),
    InlineEnum(protos::Text, Vec<Variant>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeDeclaration {
    Struct(protos::Text, Vec<TypeExpression>),
    Enum(protos::Text, Vec<Variant>),
    Alias(protos::Text, TypeExpression),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Receiver {
    Shared,
    Mutable,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub name: protos::Text,
    pub receiver: Receiver,
    pub inputs: Vec<TypeExpression>,
    pub yield_type: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedType {
    pub name: protos::Text,
    pub constraints: Vec<protos::Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedConstant {
    pub name: protos::Text,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindConstraint {
    pub bounds: Vec<protos::Text>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KindDeclaration {
    Simple {
        name: protos::Text,
        constraints: Vec<KindConstraint>,
        capabilities: Vec<Capability>,
    },
    Complex {
        name: protos::Text,
        constraints: Vec<KindConstraint>,
        superkinds: Vec<protos::Text>,
        associated_types: Vec<AssociatedType>,
        associated_constants: Vec<AssociatedConstant>,
        capabilities: Vec<Capability>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Association {
    pub ty: protos::Text,
    pub kinds: Vec<protos::Text>,
}

// ============================================================================
// Faults
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Problem {
    Protos,
    Root,
    Section,
    Import,
    Declaration,
    TypeExpression,
    Capability,
    Kind,
    Association,
    Generation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fault {
    pub path: protos::Path,
    pub problem: Problem,
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}", self.problem, self.path)
    }
}

impl std::error::Error for Fault {}

// ============================================================================
// Kinds (traits)
// ============================================================================

/// Canonicalizing: sweet ethos text → canonical braced form.
pub trait Canonicalizing {
    fn canonicalize(&self) -> protos::Text;
}

/// Generating: a File emits Rust source.
pub trait Generating {
    type Fault;
    fn generate(&self) -> Result<protos::Text, Self::Fault>;
}

// ============================================================================
// Canonicalizing: sweet → canonical text
// ============================================================================

impl Canonicalizing for str {
    fn canonicalize(&self) -> protos::Text {
        canonicalize_sweet(self)
    }
}

/// Convert sweet form to canonical form.
///
/// Sweet form: the variant head as a bare word, then sections as siblings.
/// Canonical form: `Head.{ sections }`.
///
/// If the text is already canonical (first token is headed with a dot),
/// it is returned unchanged.
fn canonicalize_sweet(source: &str) -> protos::Text {
    // Find the first non-whitespace, non-comment content
    let trimmed = skip_leading_ws_comments(source);
    if trimmed.is_empty() {
        return source.to_owned();
    }

    // Extract the first bare word
    let first_word_end = trimmed
        .find(|c: char| c.is_whitespace() || is_delimiter_char(c) || c == ';' || c == '.')
        .unwrap_or(trimmed.len());

    if first_word_end == 0 {
        // Starts with a delimiter or dot — already structured
        return source.to_owned();
    }

    let head = &trimmed[..first_word_end];
    let after_head = &trimmed[first_word_end..];

    // Check if what follows is a dot + opener (canonical form)
    let after_trimmed = after_head.trim_start();
    if after_trimmed.starts_with('.') {
        // Already canonical: Head.{ ... } or Head.[ ... ]
        return source.to_owned();
    }

    // Sweet form: wrap everything after the head in braces
    let rest = after_head;
    format!("{head}.{{ {rest} }}")
}

fn skip_leading_ws_comments(s: &str) -> &str {
    let mut pos = 0;
    let bytes = s.as_bytes();
    loop {
        // Skip whitespace
        while pos < bytes.len() && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }
        // Skip comment
        if pos < bytes.len() && bytes[pos] == b';' {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        break;
    }
    &s[pos..]
}

fn is_delimiter_char(c: char) -> bool {
    matches!(
        c,
        '{' | '}' | '[' | ']' | '<' | '>'
            | '\u{201C}' | '\u{201D}'
            | '(' | ')'
    )
}

// ============================================================================
// Conceivable<File>: Protoform → File (the reader)
// ============================================================================

impl protos::Conceivable<File> for Delineation {
    type Fault = Fault;

    fn conceive(&self) -> Result<File, Fault> {
        let pfs = &self.protoforms;
        let first = pfs.first().ok_or_else(|| fault(vec![], Problem::Root))?;

        let [pf] = pfs.as_slice() else {
            return Err(fault(vec![], Problem::Root));
        };

        let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Root))?;
        if sep != Separator::Period {
            return Err(fault(vec![], Problem::Root));
        }

        let _ = first; // used above for the ok_or_else
        let sections = pf_braced(body).ok_or_else(|| fault(vec![], Problem::Root))?;

        match head {
            "Types" => read_types_file(sections),
            "Kinds" => read_kinds_file(sections),
            "Signal" => read_signal_file(sections),
            "Sema" => read_sema_file(sections),
            _ => Err(fault(vec![], Problem::Root)),
        }
    }
}

impl protos::Conceivable<File> for Protoform {
    type Fault = Fault;

    fn conceive(&self) -> Result<File, Fault> {
        let delineation = Delineation {
            protoforms: vec![self.clone()],
            situation: protos::Situation::new(),
        };
        delineation.conceive()
    }
}

fn read_types_file(sections: &[Protoform]) -> Result<File, Fault> {
    if sections.len() != 3 {
        return Err(fault(vec![], Problem::Section));
    }
    Ok(File::Types(Types {
        imports: read_imports(&sections[0])?,
        types: read_type_declarations(&sections[1])?,
        associations: read_associations(&sections[2])?,
    }))
}

fn read_kinds_file(sections: &[Protoform]) -> Result<File, Fault> {
    if sections.len() != 2 {
        return Err(fault(vec![], Problem::Section));
    }
    Ok(File::Kinds(Kinds {
        imports: read_imports(&sections[0])?,
        kinds: read_kind_declarations(&sections[1])?,
    }))
}

fn read_signal_file(sections: &[Protoform]) -> Result<File, Fault> {
    if sections.len() != 4 {
        return Err(fault(vec![], Problem::Section));
    }
    Ok(File::Signal(Signal {
        imports: read_imports(&sections[0])?,
        requests: read_variant_list(&sections[1])?,
        responses: read_variant_list(&sections[2])?,
        types: read_type_declarations(&sections[3])?,
    }))
}

fn read_sema_file(sections: &[Protoform]) -> Result<File, Fault> {
    if sections.len() != 2 {
        return Err(fault(vec![], Problem::Section));
    }
    Ok(File::Sema(Sema {
        imports: read_imports(&sections[0])?,
        types: read_type_declarations(&sections[1])?,
    }))
}

// ---------------------------------------------------------------------------
// Reader: imports
// ---------------------------------------------------------------------------

fn read_imports(pf: &Protoform) -> Result<Vec<Import>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Import))?;
    children.iter().map(read_import).collect()
}

fn read_import(pf: &Protoform) -> Result<Import, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Import))?;
    if sep != Separator::Colon {
        return Err(fault(vec![], Problem::Import));
    }
    let source = head.to_owned();
    if let Some(names) = pf_bracketed(body) {
        let names = names
            .iter()
            .map(|p| bare_symbol(p).map(str::to_owned).ok_or_else(|| fault(vec![], Problem::Import)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Import::Multiple(source, names))
    } else {
        let name = bare_symbol(body).ok_or_else(|| fault(vec![], Problem::Import))?.to_owned();
        Ok(Import::Single(source, name))
    }
}

// ---------------------------------------------------------------------------
// Reader: type declarations
// ---------------------------------------------------------------------------

fn read_type_declarations(pf: &Protoform) -> Result<Vec<TypeDeclaration>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Declaration))?;
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
    // Bare names with no body: intrinsic type aliases (Text, Integer, etc.)
    if let Some(name) = bare_symbol(pf) {
        // A bare name in the types section is a type without a definition —
        // it's an intrinsic declaration. Emit as an alias to itself.
        return Ok((TypeDeclaration::Alias(name.to_owned(), TypeExpression::Named(name.to_owned())), false));
    }

    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Declaration))?;
    if sep != Separator::Period {
        return Err(fault(vec![], Problem::Declaration));
    }
    let name = head.to_owned();

    if let Some(children) = pf_braced(body) {
        let fields = read_type_expression_list(children)?;
        return Ok((TypeDeclaration::Struct(name, fields), false));
    }

    if let Some(children) = pf_bracketed(body) {
        let variants = read_variants(children)?;
        return Ok((TypeDeclaration::Enum(name, variants), false));
    }

    let (target, consumed) = read_type_expression_with_following(body, following)?;
    Ok((TypeDeclaration::Alias(name, target), consumed))
}

// ---------------------------------------------------------------------------
// Reader: variants
// ---------------------------------------------------------------------------

fn read_variant_list(pf: &Protoform) -> Result<Vec<Variant>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Declaration))?;
    read_variants(children)
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
            return Err(fault(vec![], Problem::Declaration));
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
        .ok_or_else(|| fault(vec![], Problem::Declaration))?
        .to_owned();
    Ok((Variant::Unit(name), false))
}

// ---------------------------------------------------------------------------
// Reader: type expressions
// ---------------------------------------------------------------------------

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
    if let Protoform::Bare(Head::Qualified(constructor, args)) = pf {
        let arguments = read_type_expression_list(args)?;
        return Ok((TypeExpression::Applied(constructor.to_owned(), arguments), false));
    }

    if let Some(name) = bare_symbol(pf) {
        if name == "Self" {
            return Ok((TypeExpression::SelfType, false));
        }
        if let Some(angled) = following.and_then(pf_angled) {
            let arguments = read_type_expression_list(angled)?;
            return Ok((TypeExpression::Applied(name.to_owned(), arguments), true));
        }
        return Ok((TypeExpression::Named(name.to_owned()), false));
    }

    Err(fault(vec![], Problem::TypeExpression))
}

fn read_single_type_expression(pfs: &[Protoform]) -> Result<TypeExpression, Fault> {
    if pfs.is_empty() {
        return Err(fault(vec![], Problem::TypeExpression));
    }
    let (expr, consumed) = read_type_expression_with_following(&pfs[0], pfs.get(1))?;
    let expected_len = 1 + usize::from(consumed);
    if pfs.len() != expected_len {
        return Err(fault(vec![], Problem::TypeExpression));
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Reader: kinds
// ---------------------------------------------------------------------------

fn read_kind_declarations(pf: &Protoform) -> Result<Vec<KindDeclaration>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
    children.iter().map(read_kind).collect()
}

fn read_kind(pf: &Protoform) -> Result<KindDeclaration, Fault> {
    let (head, _sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
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
        let constraints = if !head_constraints.is_empty() {
            head_constraints
        } else {
            Vec::new()
        };

        // Complex kind: [ superkinds ] [ associated_types ] [ constants ] [ capabilities ]
        if children.len() == 4 {
            let superkinds = read_bare_list(&children[0])?;
            let associated_types = read_associated_types(&children[1])?;
            let associated_constants = read_associated_constants(&children[2])?;
            let cap_children =
                pf_bracketed(&children[3]).ok_or_else(|| fault(vec![], Problem::Kind))?;
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

        return Err(fault(vec![], Problem::Kind));
    }

    Err(fault(vec![], Problem::Kind))
}

fn read_kind_constraints(pfs: &[Protoform]) -> Result<Vec<KindConstraint>, Fault> {
    let mut constraints = Vec::new();
    for pf in pfs {
        if let Some(children) = pf_bracketed(pf) {
            let bounds = children
                .iter()
                .map(|p| bare_symbol(p).map(str::to_owned).ok_or_else(|| fault(vec![], Problem::Kind)))
                .collect::<Result<Vec<_>, _>>()?;
            constraints.push(KindConstraint { bounds });
        } else {
            let bound = bare_symbol(pf)
                .ok_or_else(|| fault(vec![], Problem::Kind))?
                .to_owned();
            constraints.push(KindConstraint { bounds: vec![bound] });
        }
    }
    Ok(constraints)
}

fn read_bare_list(pf: &Protoform) -> Result<Vec<protos::Text>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
    children
        .iter()
        .map(|p| bare_symbol(p).map(str::to_owned).ok_or_else(|| fault(vec![], Problem::Kind)))
        .collect()
}

fn read_associated_types(pf: &Protoform) -> Result<Vec<AssociatedType>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
    let mut types = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let child = &children[index];
        // Check for Qualified head: Name<Constraint>
        if let Protoform::Bare(Head::Qualified(name, quals)) = child {
            let constraints = quals
                .iter()
                .map(|p| bare_symbol(p).map(str::to_owned).ok_or_else(|| fault(vec![], Problem::Kind)))
                .collect::<Result<Vec<_>, _>>()?;
            types.push(AssociatedType {
                name: name.clone(),
                constraints,
            });
            index += 1;
        } else {
            let name = bare_symbol(child)
                .ok_or_else(|| fault(vec![], Problem::Kind))?
                .to_owned();
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
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
    children.iter().map(read_associated_constant).collect()
}

fn read_associated_constant(pf: &Protoform) -> Result<AssociatedConstant, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Kind))?;
    if sep != Separator::Period {
        return Err(fault(vec![], Problem::Kind));
    }
    let name = head.to_owned();
    let (ty, _consumed) = read_type_expression_with_following(body, None)?;
    Ok(AssociatedConstant { name, ty })
}

fn read_capabilities(pfs: &[Protoform]) -> Result<Vec<Capability>, Fault> {
    pfs.iter().map(read_capability).collect()
}

fn read_capability(pf: &Protoform) -> Result<Capability, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Capability))?;
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
            return Err(fault(vec![], Problem::Capability));
        }
        let input_children =
            pf_bracketed(&children[0]).ok_or_else(|| fault(vec![], Problem::Capability))?;
        let inputs = read_type_expression_list(input_children)?;
        let yield_children =
            pf_bracketed(&children[1]).ok_or_else(|| fault(vec![], Problem::Capability))?;
        let yield_type = read_single_type_expression(yield_children)?;
        return Ok(Capability {
            name,
            receiver,
            inputs,
            yield_type,
        });
    }

    Err(fault(vec![], Problem::Capability))
}

// ---------------------------------------------------------------------------
// Reader: associations
// ---------------------------------------------------------------------------

fn read_associations(pf: &Protoform) -> Result<Vec<Association>, Fault> {
    let children = pf_bracketed(pf).ok_or_else(|| fault(vec![], Problem::Association))?;
    children.iter().map(read_association).collect()
}

fn read_association(pf: &Protoform) -> Result<Association, Fault> {
    let (head, sep, body) = pf_headed(pf).ok_or_else(|| fault(vec![], Problem::Association))?;
    if sep != Separator::Period {
        return Err(fault(vec![], Problem::Association));
    }
    let ty = head.to_owned();
    let kinds_children = pf_bracketed(body).ok_or_else(|| fault(vec![], Problem::Association))?;
    let kinds = kinds_children
        .iter()
        .map(|p| bare_symbol(p).map(str::to_owned).ok_or_else(|| fault(vec![], Problem::Association)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Association { ty, kinds })
}

// ============================================================================
// Protosizable: File → Protoform (concept → protoform, cannot fault)
// ============================================================================

impl protos::Protosizable for File {
    type Fault = std::convert::Infallible;

    fn protosize(&self) -> Result<Delineation, std::convert::Infallible> {
        Ok(Delineation {
            protoforms: vec![file_to_protoform(self)],
            situation: protos::Situation::new(),
        })
    }
}

fn file_to_protoform(file: &File) -> Protoform {
    match file {
        File::Types(types) => headed("Types", Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_imports(&types.imports),
                protosize_type_declarations(&types.types),
                protosize_associations(&types.associations),
            ],
        )),
        File::Kinds(kinds) => headed("Kinds", Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_imports(&kinds.imports),
                protosize_kind_declarations(&kinds.kinds),
            ],
        )),
        File::Signal(signal) => headed("Signal", Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_imports(&signal.imports),
                protosize_variants(&signal.requests),
                protosize_variants(&signal.responses),
                protosize_type_declarations(&signal.types),
            ],
        )),
        File::Sema(sema) => headed("Sema", Protoform::Enclosed(
            Enclosure::Braced,
            vec![
                protosize_imports(&sema.imports),
                protosize_type_declarations(&sema.types),
            ],
        )),
    }
}

fn headed(name: &str, body: Protoform) -> Protoform {
    Protoform::Headed(Head::Bare(name.to_owned()), Separator::Period, Box::new(body))
}

fn protosize_imports(imports: &[Import]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        imports.iter().map(protosize_import).collect(),
    )
}

fn protosize_import(import: &Import) -> Protoform {
    match import {
        Import::Single(source, name) => Protoform::Headed(
            Head::Bare(source.clone()),
            Separator::Colon,
            Box::new(Protoform::Bare(Head::Bare(name.clone()))),
        ),
        Import::Multiple(source, names) => Protoform::Headed(
            Head::Bare(source.clone()),
            Separator::Colon,
            Box::new(Protoform::Enclosed(
                Enclosure::Bracketed,
                names.iter().map(|n| Protoform::Bare(Head::Bare(n.clone()))).collect(),
            )),
        ),
    }
}

fn protosize_type_declarations(types: &[TypeDeclaration]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        types.iter().map(protosize_type_declaration).collect(),
    )
}

fn protosize_type_declaration(decl: &TypeDeclaration) -> Protoform {
    match decl {
        TypeDeclaration::Struct(name, fields) => headed(
            name,
            Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(protosize_type_expression).collect(),
            ),
        ),
        TypeDeclaration::Enum(name, variants) => headed(
            name,
            Protoform::Enclosed(
                Enclosure::Bracketed,
                variants.iter().map(protosize_variant).collect(),
            ),
        ),
        TypeDeclaration::Alias(name, target) => headed(name, protosize_type_expression(target)),
    }
}

fn protosize_variants(variants: &[Variant]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        variants.iter().map(protosize_variant).collect(),
    )
}

fn protosize_variant(variant: &Variant) -> Protoform {
    match variant {
        Variant::Unit(name) => Protoform::Bare(Head::Bare(name.clone())),
        Variant::Typed(name, ty) => headed(name, protosize_type_expression(ty)),
        Variant::InlineStruct(name, fields) => headed(
            name,
            Protoform::Enclosed(
                Enclosure::Braced,
                fields.iter().map(protosize_type_expression).collect(),
            ),
        ),
        Variant::InlineEnum(name, variants) => headed(
            name,
            Protoform::Enclosed(
                Enclosure::Bracketed,
                variants.iter().map(protosize_variant).collect(),
            ),
        ),
    }
}

fn protosize_type_expression(expr: &TypeExpression) -> Protoform {
    match expr {
        TypeExpression::Named(name) => Protoform::Bare(Head::Bare(name.clone())),
        TypeExpression::Applied(constructor, arguments) => Protoform::Bare(Head::Qualified(
            constructor.clone(),
            arguments.iter().map(protosize_type_expression).collect(),
        )),
        TypeExpression::SelfType => Protoform::Bare(Head::Bare("Self".to_owned())),
    }
}

fn protosize_kind_declarations(kinds: &[KindDeclaration]) -> Protoform {
    Protoform::Enclosed(
        Enclosure::Bracketed,
        kinds.iter().map(protosize_kind_declaration).collect(),
    )
}

fn protosize_kind_declaration(kind: &KindDeclaration) -> Protoform {
    let (name, constraints) = match kind {
        KindDeclaration::Simple { name, constraints, .. }
        | KindDeclaration::Complex { name, constraints, .. } => (name, constraints),
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
                        superkinds.iter().map(|s| Protoform::Bare(Head::Bare(s.clone()))).collect(),
                    ),
                    Protoform::Enclosed(
                        Enclosure::Bracketed,
                        associated_types.iter().map(protosize_associated_type).collect(),
                    ),
                    Protoform::Enclosed(
                        Enclosure::Bracketed,
                        associated_constants.iter().map(protosize_associated_constant).collect(),
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
        Protoform::Bare(Head::Bare(kc.bounds[0].clone()))
    } else {
        Protoform::Enclosed(
            Enclosure::Bracketed,
            kc.bounds.iter().map(|b| Protoform::Bare(Head::Bare(b.clone()))).collect(),
        )
    }
}

fn protosize_associated_type(at: &AssociatedType) -> Protoform {
    if at.constraints.is_empty() {
        Protoform::Bare(Head::Bare(at.name.clone()))
    } else {
        Protoform::Bare(Head::Qualified(
            at.name.clone(),
            at.constraints.iter().map(|c| Protoform::Bare(Head::Bare(c.clone()))).collect(),
        ))
    }
}

fn protosize_associated_constant(ac: &AssociatedConstant) -> Protoform {
    headed(&ac.name, protosize_type_expression(&ac.ty))
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
            assoc.kinds.iter().map(|k| Protoform::Bare(Head::Bare(k.clone()))).collect(),
        )),
    )
}

// ============================================================================
// Generating: File → Rust (the emitter)
// ============================================================================

impl Generating for File {
    type Fault = Fault;

    fn generate(&self) -> Result<protos::Text, Fault> {
        let tokens = emit_tokens(self)?;
        let syntax: syn::File = syn::parse2(tokens).map_err(|_| fault(vec![], Problem::Generation))?;
        Ok(syntax.into_token_stream().to_string())
    }
}

/// Build an import resolution table: name → source module.
fn build_import_resolution(imports: &[Import]) -> HashMap<String, String> {
    let mut resolution = HashMap::new();
    for import in imports {
        match import {
            Import::Single(source, name) => {
                resolution.insert(name.clone(), source.clone());
            }
            Import::Multiple(source, names) => {
                for name in names {
                    resolution.insert(name.clone(), source.clone());
                }
            }
        }
    }
    resolution
}

fn emit_tokens(file: &File) -> Result<proc_macro2::TokenStream, Fault> {
    let mut tokens = quote! { #![allow(dead_code)] };

    match file {
        File::Types(types) => {
            let imports = build_import_resolution(&types.imports);
            for ty in &types.types {
                tokens.extend(type_declaration_tokens(ty, &imports)?);
                tokens.extend(datomic_impl_tokens(ty, &imports)?);
            }
            for assoc in &types.associations {
                tokens.extend(association_assertion_tokens(assoc)?);
            }
        }
        File::Kinds(kinds) => {
            let imports = build_import_resolution(&kinds.imports);
            for kind in &kinds.kinds {
                tokens.extend(kind_declaration_tokens(kind, &imports)?);
            }
        }
        File::Signal(signal) => {
            let imports = build_import_resolution(&signal.imports);
            for ty in &signal.types {
                tokens.extend(type_declaration_tokens(ty, &imports)?);
                tokens.extend(datomic_impl_tokens(ty, &imports)?);
            }
            tokens.extend(signal_enum_tokens("Request", &signal.requests, &imports)?);
            tokens.extend(signal_enum_tokens("Response", &signal.responses, &imports)?);
        }
        File::Sema(sema) => {
            let imports = build_import_resolution(&sema.imports);
            for ty in &sema.types {
                tokens.extend(type_declaration_tokens(ty, &imports)?);
                tokens.extend(datomic_impl_tokens(ty, &imports)?);
            }
        }
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Generator: type declarations
// ---------------------------------------------------------------------------

fn all_variants_unit(variants: &[Variant]) -> bool {
    !variants.is_empty() && variants.iter().all(|v| matches!(v, Variant::Unit(_)))
}

fn enum_derive(unit_only: bool) -> proc_macro2::TokenStream {
    if unit_only {
        quote! { #[derive(Clone, Copy, Debug, PartialEq, Eq)] }
    } else {
        quote! { #[derive(Clone, Debug, PartialEq, Eq)] }
    }
}

fn type_declaration_tokens(
    decl: &TypeDeclaration,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    let derive = quote! { #[derive(Clone, Debug, PartialEq, Eq)] };

    Ok(match decl {
        TypeDeclaration::Struct(name, fields) => {
            let name = rust_ident(name)?;
            let field_tokens = fields
                .iter()
                .map(|ty| {
                    let ty = type_expression_rust(ty, imports)?;
                    Ok(quote! { pub #ty })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            quote! { #derive pub struct #name ( #( #field_tokens, )* ); }
        }
        TypeDeclaration::Enum(name, variants) => {
            let name_ident = rust_ident(name)?;
            let recursive = variants_have_recursive_ref(variants, name);
            let box_ctx = recursive.then_some(name.as_str());
            let (variant_tokens, inline_types) =
                emit_variant_decl_tokens(&name_ident, variants, imports, box_ctx)?;
            let box_impl = if recursive {
                quote! { datomic::impl_datomic_box!(#name_ident); }
            } else {
                proc_macro2::TokenStream::new()
            };
            let derive = enum_derive(all_variants_unit(variants));
            quote! {
                #( #inline_types )*
                #derive pub enum #name_ident { #( #variant_tokens, )* }
                #box_impl
            }
        }
        TypeDeclaration::Alias(name, target) => {
            // Intrinsic types (Text, Integer, etc.) that alias themselves
            // produce no output — they're already defined in protos/datomic.
            let target_tokens = type_expression_rust(target, imports)?;
            let name_tokens = rust_ident(name)?;
            // Check if this is a self-alias (intrinsic)
            if let TypeExpression::Named(tname) = target
                && tname == name && is_intrinsic(name)
            {
                return Ok(proc_macro2::TokenStream::new());
            }
            quote! { pub type #name_tokens = #target_tokens; }
        }
    })
}

fn emit_variant_decl_tokens(
    parent: &proc_macro2::Ident,
    variants: &[Variant],
    imports: &HashMap<String, String>,
    box_name: Option<&str>,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), Fault> {
    let derive = quote! { #[derive(Clone, Debug, PartialEq, Eq)] };

    let mut variant_tokens = Vec::new();
    let mut inline_types = Vec::new();

    for variant in variants {
        match variant {
            Variant::Unit(name) => {
                let name = rust_ident(name)?;
                variant_tokens.push(quote! { #name });
            }
            Variant::Typed(name, ty) => {
                let name = rust_ident(name)?;
                let ty = if let Some(bn) = box_name {
                    type_expression_rust_boxed(ty, imports, bn)?
                } else {
                    type_expression_rust(ty, imports)?
                };
                variant_tokens.push(quote! { #name(#ty) });
            }
            Variant::InlineStruct(name, fields) => {
                let variant_name = rust_ident(name)?;
                let inline_name = format_ident!("{}{}", parent, variant_name);
                let field_tokens = fields
                    .iter()
                    .map(|ty| {
                        let ty = if let Some(bn) = box_name {
                            type_expression_rust_boxed(ty, imports, bn)?
                        } else {
                            type_expression_rust(ty, imports)?
                        };
                        Ok(quote! { pub #ty })
                    })
                    .collect::<Result<Vec<_>, Fault>>()?;
                inline_types
                    .push(quote! { #derive pub struct #inline_name ( #( #field_tokens, )* ); });
                variant_tokens.push(quote! { #variant_name(#inline_name) });
            }
            Variant::InlineEnum(name, inner_variants) => {
                let variant_name = rust_ident(name)?;
                let inline_name = format_ident!("{}{}", parent, variant_name);
                let (inner_variant_tokens, inner_inline_types) =
                    emit_variant_decl_tokens(&inline_name, inner_variants, imports, box_name)?;
                inline_types.extend(inner_inline_types);
                let inline_derive = enum_derive(all_variants_unit(inner_variants));
                inline_types.push(
                    quote! { #inline_derive pub enum #inline_name { #( #inner_variant_tokens, )* } },
                );
                variant_tokens.push(quote! { #variant_name(#inline_name) });
            }
        }
    }

    Ok((variant_tokens, inline_types))
}

// ---------------------------------------------------------------------------
// Generator: Datomic impls for declared types
// ---------------------------------------------------------------------------

fn datomic_impl_tokens(
    decl: &TypeDeclaration,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    datomic_impl_tokens_inner(decl, imports, None)
}

fn datomic_impl_tokens_inner(
    decl: &TypeDeclaration,
    imports: &HashMap<String, String>,
    box_name: Option<&str>,
) -> Result<proc_macro2::TokenStream, Fault> {
    match decl {
        TypeDeclaration::Alias(name, target) => {
            // Self-alias intrinsics need no impl
            if let TypeExpression::Named(tname) = target
                && tname == name && is_intrinsic(name)
            {
                return Ok(proc_macro2::TokenStream::new());
            }
            // Type aliases carry no separate Datomic impl
            Ok(proc_macro2::TokenStream::new())
        }
        TypeDeclaration::Struct(name, fields) => {
            let name_ident = rust_ident(name)?;
            let arity = fields.len();
            let arity_i64 = arity as i64;

            let field_incorporates = fields
                .iter()
                .map(|ty| {
                    let ty = if let Some(bn) = box_name {
                        type_expression_rust_boxed(ty, imports, bn)?
                    } else {
                        type_expression_rust(ty, imports)?
                    };
                    Ok(quote! { <#ty as datomic::Datomic>::incorporate_from(iter.next().unwrap())? })
                })
                .collect::<Result<Vec<_>, Fault>>()?;

            let field_conceives = (0..fields.len()).map(|i| {
                let idx = syn::Index::from(i);
                quote! {
                    protos::Conceivable::<datomic::Datom>::conceive(&self.#idx)
                        .unwrap_or_else(|e| match e {})
                }
            });

            Ok(quote! {
                impl protos::Conceivable<datomic::Datom> for #name_ident {
                    type Fault = std::convert::Infallible;
                    fn conceive(&self) -> std::result::Result<datomic::Datom, std::convert::Infallible> {
                        Ok(datomic::Datom::Struct(vec![ #( #field_conceives, )* ]))
                    }
                }
                impl datomic::Datomic for #name_ident {
                    fn incorporate_from(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                        match datom {
                            datomic::Datom::Struct(fields) if fields.len() == #arity => {
                                let mut iter = fields.into_iter();
                                Ok(Self( #( #field_incorporates, )* ))
                            }
                            datomic::Datom::Struct(fields) => {
                                Err(datomic::Fault::Corporate(vec![], datomic::Problem::Arity(#arity_i64, fields.len() as protos::Integer)))
                            }
                            other => Err(datomic::Fault::Corporate(vec![], datomic::Problem::Shape(datomic::Expected::Struct, other))),
                        }
                    }
                }
                impl protos::Incorporable<#name_ident> for datomic::Datom {
                    type Fault = datomic::Fault;
                    fn incorporate(self) -> std::result::Result<#name_ident, datomic::Fault> {
                        <#name_ident as datomic::Datomic>::incorporate_from(self)
                    }
                }
            })
        }
        TypeDeclaration::Enum(name, variants) => {
            let name_ident = rust_ident(name)?;
            let recursive_boxing: Option<&str> = if variants_have_recursive_ref(variants, name) {
                Some(name.as_str())
            } else {
                box_name
            };

            let incorporate_arms = variants
                .iter()
                .map(|v| variant_incorporate_arm(&name_ident, v, imports, recursive_boxing))
                .collect::<Result<Vec<_>, _>>()?;
            let conceive_arms = variants
                .iter()
                .map(|v| variant_conceive_arm(&name_ident, v))
                .collect::<Result<Vec<_>, _>>()?;
            let nested = variants
                .iter()
                .filter_map(|v| {
                    nested_datomic_impl(&name_ident, name, v, imports, recursive_boxing).transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(quote! {
                impl protos::Conceivable<datomic::Datom> for #name_ident {
                    type Fault = std::convert::Infallible;
                    fn conceive(&self) -> std::result::Result<datomic::Datom, std::convert::Infallible> {
                        Ok(match self {
                            #( #conceive_arms )*
                        })
                    }
                }
                impl datomic::Datomic for #name_ident {
                    fn incorporate_from(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                        match datom {
                            #( #incorporate_arms )*
                            other => Err(datomic::Fault::Corporate(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                        }
                    }
                }
                impl protos::Incorporable<#name_ident> for datomic::Datom {
                    type Fault = datomic::Fault;
                    fn incorporate(self) -> std::result::Result<#name_ident, datomic::Fault> {
                        <#name_ident as datomic::Datomic>::incorporate_from(self)
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
            let name = rust_ident(name)?;
            quote! {
                datomic::Datom::Bare(ref s) if s == stringify!(#name) => Ok(Self::#name),
            }
        }
        Variant::Typed(name, ty) => {
            let variant_name = rust_ident(name)?;
            let recursive = box_name.is_some_and(|bn| is_direct_recursive(ty, bn));
            let inner_ty = type_expression_rust(ty, imports)?;
            if recursive {
                quote! {
                    datomic::Datom::Variant(ref head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                        Ok(Self::#variant_name(Box::new(<#inner_ty as datomic::Datomic>::incorporate_from(*body)?)))
                    }
                }
            } else {
                quote! {
                    datomic::Datom::Variant(ref head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                        Ok(Self::#variant_name(<#inner_ty as datomic::Datomic>::incorporate_from(*body)?))
                    }
                }
            }
        }
        Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = rust_ident(name)?;
            let inline_name = format_ident!("{}{}", parent, variant_name);
            quote! {
                datomic::Datom::Variant(ref head, protos::Separator::Period, Some(body)) if head == stringify!(#variant_name) => {
                    Ok(Self::#variant_name(<#inline_name as datomic::Datomic>::incorporate_from(*body)?))
                }
            }
        }
    })
}

fn variant_conceive_arm(
    _parent: &proc_macro2::Ident,
    variant: &Variant,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match variant {
        Variant::Unit(name) => {
            let name = rust_ident(name)?;
            quote! {
                Self::#name => datomic::Datom::Bare(stringify!(#name).to_owned()),
            }
        }
        Variant::Typed(name, _) | Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = rust_ident(name)?;
            quote! {
                Self::#variant_name(value) => datomic::Datom::Variant(
                    stringify!(#variant_name).to_owned(),
                    protos::Separator::Period,
                    Some(Box::new(
                        protos::Conceivable::<datomic::Datom>::conceive(value)
                            .unwrap_or_else(|e| match e {}),
                    )),
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
            Ok(Some(datomic_impl_tokens_inner(
                &TypeDeclaration::Struct(inline_name, fields.clone()),
                imports,
                box_name,
            )?))
        }
        Variant::InlineEnum(vname, inner) => {
            let inline_name = format!("{}{}", parent_name, vname);
            Ok(Some(datomic_impl_tokens_inner(
                &TypeDeclaration::Enum(inline_name, inner.clone()),
                imports,
                box_name,
            )?))
        }
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Generator: kind declarations
// ---------------------------------------------------------------------------

fn kind_declaration_tokens(
    kind: &KindDeclaration,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    let (name, constraints, superkinds, associated_types, associated_constants, capabilities) =
        match kind {
            KindDeclaration::Simple { name, constraints, capabilities } => (
                name, constraints.as_slice(), &[][..], &[][..], &[][..], capabilities.as_slice(),
            ),
            KindDeclaration::Complex {
                name, constraints, superkinds, associated_types, associated_constants, capabilities,
            } => (
                name, constraints.as_slice(), superkinds.as_slice(),
                associated_types.as_slice(), associated_constants.as_slice(), capabilities.as_slice(),
            ),
        };

    let name = rust_ident(name)?;

    let generic_params = if constraints.is_empty() {
        proc_macro2::TokenStream::new()
    } else {
        let params = constraints
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let cname = format_ident!("{}", (b'A' + i as u8) as char);
                let bounds = c.bounds.iter().map(|b| rust_ident(b)).collect::<Result<Vec<_>, _>>()?;
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
        let bounds = superkinds.iter().map(|s| rust_ident(s)).collect::<Result<Vec<_>, _>>()?;
        quote! { : #( #bounds )+* }
    };

    let associated_type_tokens = associated_types
        .iter()
        .map(|at| {
            let aname = rust_ident(&at.name)?;
            if at.constraints.is_empty() {
                Ok(quote! { type #aname; })
            } else {
                let bounds = at.constraints.iter().map(|b| rust_ident(b)).collect::<Result<Vec<_>, _>>()?;
                Ok(quote! { type #aname: #( #bounds )+*; })
            }
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let constant_tokens = associated_constants
        .iter()
        .map(|ac| {
            let cname = rust_ident(&ac.name)?;
            let ty = type_expression_rust(&ac.ty, imports)?;
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
    let return_type = type_expression_rust(&cap.yield_type, imports)?;

    let receiver = match cap.receiver {
        Receiver::Shared => quote! { &self },
        Receiver::Mutable => quote! { &mut self },
        Receiver::None => proc_macro2::TokenStream::new(),
    };

    let input_params = if cap.inputs.len() == 1 {
        let ty = type_expression_rust(&cap.inputs[0], imports)?;
        vec![quote! { input: #ty }]
    } else {
        cap.inputs
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let pname = format_ident!("input_{}", i);
                let ty = type_expression_rust(ty, imports)?;
                Ok(quote! { #pname: #ty })
            })
            .collect::<Result<Vec<_>, Fault>>()?
    };

    let params = if receiver.is_empty() {
        quote! { #( #input_params ),* }
    } else if input_params.is_empty() {
        receiver
    } else {
        quote! { #receiver, #( #input_params ),* }
    };

    Ok(quote! { fn #name( #params ) -> #return_type; })
}

// ---------------------------------------------------------------------------
// Generator: associations
// ---------------------------------------------------------------------------

fn association_assertion_tokens(assoc: &Association) -> Result<proc_macro2::TokenStream, Fault> {
    let ty = rust_ident(&assoc.ty)?;
    let assertions = assoc
        .kinds
        .iter()
        .map(|kind_name| {
            let assertion_fn = format_ident!(
                "assert_{}_{}",
                assoc.ty.to_lowercase(),
                kind_name.to_lowercase()
            );
            let kind = rust_ident(kind_name)?;
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

// ---------------------------------------------------------------------------
// Generator: signal enum generation
// ---------------------------------------------------------------------------

fn signal_enum_tokens(
    name: &str,
    variants: &[Variant],
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    if variants.is_empty() {
        return Ok(proc_macro2::TokenStream::new());
    }
    let enum_name = rust_ident(name)?;
    let derive = enum_derive(all_variants_unit(variants));

    let (variant_tokens, inline_types) =
        emit_variant_decl_tokens(&enum_name, variants, imports, None)?;

    let incorporate_arms = variants
        .iter()
        .map(|v| variant_incorporate_arm(&enum_name, v, imports, None))
        .collect::<Result<Vec<_>, Fault>>()?;
    let conceive_arms = variants
        .iter()
        .map(|v| variant_conceive_arm(&enum_name, v))
        .collect::<Result<Vec<_>, Fault>>()?;
    let nested = variants
        .iter()
        .filter_map(|v| {
            nested_datomic_impl(&enum_name, name, v, imports, None).transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #( #inline_types )*
        #derive pub enum #enum_name { #( #variant_tokens, )* }
        impl protos::Conceivable<datomic::Datom> for #enum_name {
            type Fault = std::convert::Infallible;
            fn conceive(&self) -> std::result::Result<datomic::Datom, std::convert::Infallible> {
                Ok(match self {
                    #( #conceive_arms )*
                })
            }
        }
        impl datomic::Datomic for #enum_name {
            fn incorporate_from(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
                match datom {
                    #( #incorporate_arms )*
                    other => Err(datomic::Fault::Corporate(vec![], datomic::Problem::Shape(datomic::Expected::Variant, other))),
                }
            }
        }
        impl protos::Incorporable<#enum_name> for datomic::Datom {
            type Fault = datomic::Fault;
            fn incorporate(self) -> std::result::Result<#enum_name, datomic::Fault> {
                <#enum_name as datomic::Datomic>::incorporate_from(self)
            }
        }
        #( #nested )*
    })
}

// ============================================================================
// Type expression → Rust tokens
// ============================================================================

fn type_expression_rust(
    expr: &TypeExpression,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match expr {
        TypeExpression::Named(name) => resolve_type_name(name, imports)?,
        TypeExpression::Applied(constructor, arguments) => {
            let args = arguments
                .iter()
                .map(|a| type_expression_rust(a, imports))
                .collect::<Result<Vec<_>, _>>()?;
            match constructor.as_str() {
                "Vector" => {
                    let [inner] = args.as_slice() else {
                        return Err(fault(vec![], Problem::Generation));
                    };
                    quote! { Vec<#inner> }
                }
                "Option" => {
                    let [inner] = args.as_slice() else {
                        return Err(fault(vec![], Problem::Generation));
                    };
                    quote! { Option<#inner> }
                }
                "Result" => {
                    let [ok, err] = args.as_slice() else {
                        return Err(fault(vec![], Problem::Generation));
                    };
                    quote! { Result<#ok, #err> }
                }
                "Box" => {
                    let [inner] = args.as_slice() else {
                        return Err(fault(vec![], Problem::Generation));
                    };
                    quote! { Box<#inner> }
                }
                _ => {
                    if let Some(module) = imports.get(constructor.as_str()) {
                        let module = rust_ident(module)?;
                        let name = rust_ident(constructor)?;
                        quote! { #module :: #name < #( #args ),* > }
                    } else {
                        let name = rust_ident(constructor)?;
                        quote! { #name< #( #args ),* > }
                    }
                }
            }
        }
        TypeExpression::SelfType => quote! { Self },
    })
}

fn resolve_type_name(
    name: &str,
    imports: &HashMap<String, String>,
) -> Result<proc_macro2::TokenStream, Fault> {
    // Intrinsic names: always fully qualified
    match name {
        "Text" | "Integer" | "Decimal" | "Boolean" | "Symbol" => {
            let ident = rust_ident(name)?;
            return Ok(quote! { protos::#ident });
        }
        "Meaning" => {
            return Ok(quote! { datomic::Meaning });
        }
        _ => {}
    }
    // Imported names: fully qualified
    if let Some(module) = imports.get(name) {
        let module = rust_ident(module)?;
        let name = rust_ident(name)?;
        Ok(quote! { #module :: #name })
    } else {
        let name = rust_ident(name)?;
        Ok(quote! { #name })
    }
}

fn is_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "Text" | "Integer" | "Decimal" | "Boolean" | "Meaning"
            | "Symbol" | "Vector" | "Option" | "Result" | "Self"
    )
}

// ============================================================================
// Recursive-type helpers
// ============================================================================

fn is_direct_recursive(ty: &TypeExpression, enclosing: &str) -> bool {
    matches!(ty, TypeExpression::Named(n) if n == enclosing)
}

fn fields_have_recursive_ref(fields: &[TypeExpression], enclosing: &str) -> bool {
    fields.iter().any(|f| is_direct_recursive(f, enclosing))
}

fn variants_have_recursive_ref(variants: &[Variant], enclosing: &str) -> bool {
    variants.iter().any(|v| match v {
        Variant::Unit(_) => false,
        Variant::Typed(_, ty) => is_direct_recursive(ty, enclosing),
        Variant::InlineStruct(_, fields) => fields_have_recursive_ref(fields, enclosing),
        Variant::InlineEnum(_, inner) => variants_have_recursive_ref(inner, enclosing),
    })
}

fn type_expression_rust_boxed(
    expr: &TypeExpression,
    imports: &HashMap<String, String>,
    box_name: &str,
) -> Result<proc_macro2::TokenStream, Fault> {
    if is_direct_recursive(expr, box_name) {
        let inner = type_expression_rust(expr, imports)?;
        return Ok(quote! { Box<#inner> });
    }
    type_expression_rust(expr, imports)
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

fn bare_symbol(pf: &Protoform) -> Option<&str> {
    match pf {
        Protoform::Bare(Head::Bare(s)) => Some(s.as_str()),
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

fn pf_angled(pf: &Protoform) -> Option<&[Protoform]> {
    pf_enclosed(pf, Enclosure::Angled)
}

// ============================================================================
// Fault and ident helpers
// ============================================================================

fn fault(path: protos::Path, problem: Problem) -> Fault {
    Fault { path, problem }
}

fn rust_ident(name: &str) -> Result<proc_macro2::Ident, Fault> {
    Ok(format_ident!("{}", name))
}
