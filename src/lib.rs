use std::fmt;

use protos::{Delineatable, EnclosedAnatomy, Extent, Portion, Separator, StructuralEnclosure};
use quote::{ToTokens, format_ident, quote};

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
// Potential and kinds
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
            self.problem, self.extent.start, self.extent.end
        )
    }
}

impl std::error::Error for Fault {}

// ============================================================================
// Reader (Actualizing for Potential)
// ============================================================================

impl Actualizing for Potential {
    fn actualize(&self) -> Result<Concept, Fault> {
        let text = protos::Text::<()>::from(self.0.as_str());
        let delineation = text.delineate().map_err(|f| Fault {
            extent: f.extent,
            problem: Problem::Protos,
        })?;
        let portions = &delineation.portions;

        let first = portions.first().ok_or_else(|| root_fault(self.0.len()))?;

        let headed = portion_headed(first).ok_or_else(|| fault_at(first, Problem::Root))?;
        if headed.separator != Separator::Period {
            return Err(fault_at(first, Problem::Root));
        }

        match headed.head.as_ref() {
            "Library" => read_library(&headed.body, &portions[1..]),
            "Signal" => read_signal(&headed.body, &portions[1..]),
            _ => Err(fault_at(first, Problem::Root)),
        }
    }
}

fn read_library(version_or_body: &Portion, rest: &[Portion]) -> Result<Concept, Fault> {
    let (version, sections) = extract_version_and_sections(version_or_body, rest, 4)?;
    Ok(Concept::Library(Library {
        version,
        imports: read_imports(sections[0])?,
        types: read_types(sections[1])?,
        kinds: read_kinds(sections[2])?,
        associations: read_associations(sections[3])?,
    }))
}

fn read_signal(version_or_body: &Portion, rest: &[Portion]) -> Result<Concept, Fault> {
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
    body: &'a Portion,
    rest: &'a [Portion],
    expected: usize,
) -> Result<(Version, Vec<&'a Portion>), Fault> {
    let braced = portion_braced(body).ok_or_else(|| fault_at(body, Problem::Version))?;

    if braced.iter().all(|p| portion_bare(p).is_some()) {
        let version = read_version(braced)?;
        if rest.len() != expected {
            return Err(fault_at(body, Problem::Section));
        }
        Ok((version, rest.iter().collect()))
    } else {
        if braced.len() != 1 + expected {
            return Err(fault_at(body, Problem::Section));
        }
        let version_children =
            portion_braced(&braced[0]).ok_or_else(|| fault_at(&braced[0], Problem::Version))?;
        let version = read_version(version_children)?;
        Ok((version, braced[1..].iter().collect()))
    }
}

fn read_version(portions: &[Portion]) -> Result<Version, Fault> {
    let [major, minor, patch] = portions else {
        return Err(Fault {
            extent: Extent { start: 0, end: 0 },
            problem: Problem::Version,
        });
    };
    Ok(Version(
        bare_integer(major)?,
        bare_integer(minor)?,
        bare_integer(patch)?,
    ))
}

fn read_imports(portion: &Portion) -> Result<Vec<Import>, Fault> {
    let children = portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Import))?;
    children.iter().map(read_import).collect()
}

fn read_import(portion: &Portion) -> Result<Import, Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Import))?;
    if headed.separator != Separator::Colon {
        return Err(fault_at(portion, Problem::Import));
    }
    let source = headed.head.as_ref().to_owned();
    if let Some(names) = portion_bracketed(&headed.body) {
        let names = names
            .iter()
            .map(|p| {
                bare_symbol(p)
                    .map(str::to_owned)
                    .map_err(|()| fault_at(p, Problem::Import))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Import::Multiple { source, names })
    } else {
        let name = bare_symbol(&headed.body)
            .map_err(|()| fault_at(portion, Problem::Import))?
            .to_owned();
        Ok(Import::Single { source, name })
    }
}

fn read_types(portion: &Portion) -> Result<Vec<TypeDeclaration>, Fault> {
    let children =
        portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Declaration))?;
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
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeDeclaration, bool), Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Declaration))?;
    if headed.separator != Separator::Period {
        return Err(fault_at(portion, Problem::Declaration));
    }
    let name = headed.head.as_ref().to_owned();
    let body = &*headed.body;

    if let Some(children) = portion_braced(body) {
        let fields = read_type_expression_list(children)?;
        return Ok((TypeDeclaration::Struct { name, fields }, false));
    }

    if let Some(children) = portion_bracketed(body) {
        let variants = read_variants(children)?;
        return Ok((TypeDeclaration::Enum { name, variants }, false));
    }

    if let Some(children) = portion_guillemets(body) {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < children.len() {
            let (expr, ate) =
                read_type_expression_with_following(&children[i], children.get(i + 1))?;
            exprs.push(expr);
            i += 1 + usize::from(ate);
        }
        if exprs.len() != 2 {
            return Err(fault_at(body, Problem::Declaration));
        }
        let value = exprs.pop().unwrap();
        let key = exprs.pop().unwrap();
        return Ok((TypeDeclaration::Map { name, key, value }, false));
    }

    let (target, consumed) = read_type_expression_with_following(body, following)?;
    Ok((TypeDeclaration::Alias { name, target }, consumed))
}

fn read_variants(portions: &[Portion]) -> Result<Vec<Variant>, Fault> {
    let mut variants = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let (variant, consumed) = read_variant(&portions[index], portions.get(index + 1))?;
        variants.push(variant);
        index += 1 + usize::from(consumed);
    }
    Ok(variants)
}

fn read_variant(portion: &Portion, following: Option<&Portion>) -> Result<(Variant, bool), Fault> {
    if let Some(headed) = portion_headed(portion) {
        if headed.separator != Separator::Period {
            return Err(fault_at(portion, Problem::Declaration));
        }
        let name = headed.head.as_ref().to_owned();
        let body = &*headed.body;

        if let Some(children) = portion_braced(body) {
            let fields = read_type_expression_list(children)?;
            return Ok((Variant::InlineStruct(name, fields), false));
        }

        if let Some(children) = portion_bracketed(body) {
            let inner = read_variants(children)?;
            return Ok((Variant::InlineEnum(name, inner), false));
        }

        let (ty, consumed) = read_type_expression_with_following(body, following)?;
        return Ok((Variant::Typed(name, ty), consumed));
    }

    let name = bare_symbol(portion)
        .map_err(|()| fault_at(portion, Problem::Declaration))?
        .to_owned();
    Ok((Variant::Unit(name), false))
}

fn read_type_expression_list(portions: &[Portion]) -> Result<Vec<TypeExpression>, Fault> {
    let mut expressions = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let (expr, consumed) =
            read_type_expression_with_following(&portions[index], portions.get(index + 1))?;
        expressions.push(expr);
        index += 1 + usize::from(consumed);
    }
    Ok(expressions)
}

fn read_type_expression_with_following(
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeExpression, bool), Fault> {
    if let Ok(name) = bare_symbol(portion) {
        if name == "Self" {
            return Ok((TypeExpression::SelfType, false));
        }
        if let Some(angled) = following.and_then(portion_angled) {
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

    Err(fault_at(portion, Problem::TypeExpression))
}

fn read_kinds(portion: &Portion) -> Result<Vec<KindDeclaration>, Fault> {
    let children = portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Kind))?;
    children.iter().map(read_kind).collect()
}

fn read_kind(portion: &Portion) -> Result<KindDeclaration, Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Kind))?;
    let name = headed.head.as_ref().to_owned();
    let body = &*headed.body;

    if let Some(children) = portion_bracketed(body) {
        let capabilities = read_capabilities(children)?;
        return Ok(KindDeclaration::Simple {
            name,
            constraints: Vec::new(),
            capabilities,
        });
    }

    if let Some(children) = portion_braced(body) {
        let (constraints, body_start) =
            if let Some(angled) = children.first().and_then(portion_angled) {
                (read_kind_constraints(angled)?, 1)
            } else {
                (Vec::new(), 0)
            };

        let body_children = &children[body_start..];

        if body_children.len() == 4 {
            let superkinds = read_bare_list(&body_children[0])?;
            let associated_types = read_associated_types(&body_children[1])?;
            let associated_constants = read_associated_constants(&body_children[2])?;
            let cap_children = portion_bracketed(&body_children[3])
                .ok_or_else(|| fault_at(&body_children[3], Problem::Kind))?;
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
            && let Some(cap_children) = portion_bracketed(&body_children[0])
        {
            let capabilities = read_capabilities(cap_children)?;
            return Ok(KindDeclaration::Simple {
                name,
                constraints,
                capabilities,
            });
        }

        return Err(fault_at(body, Problem::Kind));
    }

    Err(fault_at(portion, Problem::Kind))
}

fn read_kind_constraints(portions: &[Portion]) -> Result<Vec<KindConstraint>, Fault> {
    let mut constraints = Vec::new();
    for portion in portions {
        if let Some(children) = portion_bracketed(portion) {
            let bounds = children
                .iter()
                .map(|p| {
                    bare_symbol(p)
                        .map(str::to_owned)
                        .map_err(|()| fault_at(p, Problem::Kind))
                })
                .collect::<Result<Vec<_>, _>>()?;
            constraints.push(KindConstraint {
                name: String::new(),
                bounds,
            });
        } else {
            let bound = bare_symbol(portion)
                .map_err(|()| fault_at(portion, Problem::Kind))?
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

fn read_bare_list(portion: &Portion) -> Result<Vec<String>, Fault> {
    let children = portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Kind))?;
    children
        .iter()
        .map(|p| {
            bare_symbol(p)
                .map(str::to_owned)
                .map_err(|()| fault_at(p, Problem::Kind))
        })
        .collect()
}

fn read_associated_types(portion: &Portion) -> Result<Vec<AssociatedType>, Fault> {
    let children = portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Kind))?;
    let mut types = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let child = &children[index];
        let name = bare_symbol(child)
            .map_err(|()| fault_at(child, Problem::Kind))?
            .to_owned();
        if let Some(angled) = children.get(index + 1).and_then(portion_angled) {
            let constraints = angled
                .iter()
                .map(|p| {
                    bare_symbol(p)
                        .map(str::to_owned)
                        .map_err(|()| fault_at(p, Problem::Kind))
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

fn read_associated_constants(portion: &Portion) -> Result<Vec<AssociatedConstant>, Fault> {
    let children = portion_guillemets(portion).ok_or_else(|| fault_at(portion, Problem::Kind))?;
    let mut constants = Vec::new();
    let mut index = 0;
    while index < children.len() {
        let name = bare_symbol(&children[index])
            .map_err(|()| fault_at(&children[index], Problem::Kind))?
            .to_owned();
        index += 1;
        if index >= children.len() {
            return Err(fault_at(portion, Problem::Kind));
        }
        let (ty, consumed) =
            read_type_expression_with_following(&children[index], children.get(index + 1))?;
        constants.push(AssociatedConstant { name, ty });
        index += 1 + usize::from(consumed);
    }
    Ok(constants)
}

fn read_capabilities(portions: &[Portion]) -> Result<Vec<Capability>, Fault> {
    portions.iter().map(read_capability).collect()
}

fn read_capability(portion: &Portion) -> Result<Capability, Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Capability))?;
    let name = headed.head.as_ref().to_owned();
    let receiver = match headed.separator {
        Separator::Period => Receiver::Shared,
        Separator::Exclamation => Receiver::Mutable,
        Separator::Colon => Receiver::None,
    };
    let body = &*headed.body;

    if let Some(children) = portion_bracketed(body) {
        let yield_type = read_single_type_expression(children)?;
        return Ok(Capability {
            name,
            receiver,
            inputs: Vec::new(),
            yield_type,
        });
    }

    if let Some(children) = portion_braced(body) {
        if children.len() != 2 {
            return Err(fault_at(body, Problem::Capability));
        }
        let input_children = portion_bracketed(&children[0])
            .ok_or_else(|| fault_at(&children[0], Problem::Capability))?;
        let inputs = read_type_expression_list(input_children)?;
        let yield_children = portion_bracketed(&children[1])
            .ok_or_else(|| fault_at(&children[1], Problem::Capability))?;
        let yield_type = read_single_type_expression(yield_children)?;
        return Ok(Capability {
            name,
            receiver,
            inputs,
            yield_type,
        });
    }

    Err(fault_at(body, Problem::Capability))
}

fn read_single_type_expression(portions: &[Portion]) -> Result<TypeExpression, Fault> {
    if portions.is_empty() {
        return Err(emit_fault());
    }
    let (expr, consumed) = read_type_expression_with_following(&portions[0], portions.get(1))?;
    let expected_len = 1 + usize::from(consumed);
    if portions.len() != expected_len {
        return Err(fault_at(&portions[0], Problem::TypeExpression));
    }
    Ok(expr)
}

fn read_associations(portion: &Portion) -> Result<Vec<Association>, Fault> {
    let children =
        portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Association))?;
    children.iter().map(read_association).collect()
}

fn read_association(portion: &Portion) -> Result<Association, Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Association))?;
    if headed.separator != Separator::Period {
        return Err(fault_at(portion, Problem::Association));
    }
    let ty = headed.head.as_ref().to_owned();
    let kinds_children =
        portion_bracketed(&headed.body).ok_or_else(|| fault_at(portion, Problem::Association))?;
    let kinds = kinds_children
        .iter()
        .map(|p| {
            bare_symbol(p)
                .map(str::to_owned)
                .map_err(|()| fault_at(p, Problem::Association))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Association { ty, kinds })
}

fn read_section_references(portion: &Portion) -> Result<Vec<SectionReference>, Fault> {
    let children = portion_bracketed(portion).ok_or_else(|| fault_at(portion, Problem::Section))?;
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
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(SectionReference, bool), Fault> {
    let headed = portion_headed(portion).ok_or_else(|| fault_at(portion, Problem::Section))?;
    if headed.separator != Separator::Period {
        return Err(fault_at(portion, Problem::Section));
    }
    let name = headed.head.as_ref().to_owned();
    let (ty, consumed) = read_type_expression_with_following(&headed.body, following)?;
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

    match concept {
        Concept::Library(library) => {
            for ty in &library.types {
                tokens.extend(type_declaration_tokens(ty, false)?);
                tokens.extend(datomic_impl_tokens(ty)?);
            }
            for kind in &library.kinds {
                tokens.extend(kind_declaration_tokens(kind)?);
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
                tokens.extend(type_declaration_tokens(ty, true)?);
                tokens.extend(datomic_impl_tokens(ty)?);
            }
            tokens.extend(section_enum_tokens("Request", &signal.requests, true)?);
            tokens.extend(section_enum_tokens("Reply", &signal.responses, true)?);
            tokens.extend(wire_envelope_tokens(signal)?);
        }
    }

    Ok(tokens)
}

fn type_declaration_tokens(
    decl: &TypeDeclaration,
    signal: bool,
) -> Result<proc_macro2::TokenStream, Fault> {
    let derive = if signal {
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
    } else {
        proc_macro2::TokenStream::new()
    };

    Ok(match decl {
        TypeDeclaration::Struct { name, fields } => {
            let name = ident(name)?;
            let field_tokens = fields
                .iter()
                .map(|ty| {
                    let ty = type_expression_tokens(ty)?;
                    Ok(quote! { pub #ty })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            quote! { #derive pub struct #name ( #( #field_tokens, )* ); }
        }
        TypeDeclaration::Enum { name, variants } => {
            let name_ident = ident(name)?;
            let (variant_tokens, inline_types) =
                emit_variant_tokens(&name_ident, variants, signal)?;
            quote! {
                #( #inline_types )*
                #derive pub enum #name_ident { #( #variant_tokens, )* }
            }
        }
        TypeDeclaration::Alias { name, target } => {
            if signal {
                let name = ident(name)?;
                let target = type_expression_tokens(target)?;
                quote! { #derive pub struct #name ( pub #target ); }
            } else {
                let name = ident(name)?;
                let target = type_expression_tokens(target)?;
                quote! { pub type #name = #target; }
            }
        }
        TypeDeclaration::Map { name, key, value } => {
            let name = ident(name)?;
            let key = type_expression_tokens(key)?;
            let value = type_expression_tokens(value)?;
            quote! { pub type #name = std::collections::BTreeMap<#key, #value>; }
        }
    })
}

fn emit_variant_tokens(
    parent: &proc_macro2::Ident,
    variants: &[Variant],
    signal: bool,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), Fault> {
    let derive = if signal {
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] }
    } else {
        proc_macro2::TokenStream::new()
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
                let ty = type_expression_tokens(ty)?;
                variant_tokens.push(quote! { #name(#ty) });
            }
            Variant::InlineStruct(name, fields) => {
                let variant_name = ident(name)?;
                let inline_name = format_ident!("{}{}", parent, variant_name);
                let field_tokens = fields
                    .iter()
                    .map(|ty| {
                        let ty = type_expression_tokens(ty)?;
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
                    emit_variant_tokens(&inline_name, inner_variants, signal)?;
                inline_types.extend(inner_inline_types);
                inline_types.push(
                    quote! { #derive pub enum #inline_name { #( #inner_variant_tokens, )* } },
                );
                variant_tokens.push(quote! { #variant_name(#inline_name) });
            }
        }
    }

    Ok((variant_tokens, inline_types))
}

fn type_expression_tokens(expr: &TypeExpression) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match expr {
        TypeExpression::Named(name) => match name.as_str() {
            "Text" => quote! { protos::Text },
            "Integer" => quote! { protos::Integer },
            "Decimal" => quote! { protos::Decimal },
            "Boolean" => quote! { protos::Boolean },
            "Meaning" => quote! { datomic::Meaning },
            "Symbol" => quote! { protos::Symbol },
            _ => {
                let name = ident(name)?;
                quote! { #name }
            }
        },
        TypeExpression::Applied {
            constructor,
            arguments,
        } => {
            let args = arguments
                .iter()
                .map(type_expression_tokens)
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
                _ => {
                    let name = ident(constructor)?;
                    quote! { #name< #( #args ),* > }
                }
            }
        }
        TypeExpression::SelfType => quote! { Self },
    })
}

fn datomic_impl_tokens(decl: &TypeDeclaration) -> Result<proc_macro2::TokenStream, Fault> {
    match decl {
        TypeDeclaration::Alias { name, target } => {
            let name = ident(name)?;
            let target = type_expression_tokens(target)?;
            Ok(quote! {
                impl datomic::Datomic for #name {
                    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
                        Ok(Self(<#target as datomic::Datomic>::embody(portion)?))
                    }
                    fn portion(&self) -> protos::Portion {
                        <#target as datomic::Datomic>::portion(&self.0)
                    }
                }
            })
        }
        TypeDeclaration::Map { .. } => Ok(proc_macro2::TokenStream::new()),
        TypeDeclaration::Struct { name, fields } => {
            let name = ident(name)?;
            let arity = fields.len();
            let embodies = fields
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let ty = type_expression_tokens(ty)?;
                    Ok(quote! { <#ty as datomic::Datomic>::embody(&parts[#i])? })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            let portions = (0..fields.len()).map(|i| {
                let idx = syn::Index::from(i);
                quote! { datomic::Datomic::portion(&self.#idx) }
            });
            Ok(quote! {
                impl datomic::Datomic for #name {
                    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
                        let Some(parts) = datomic::PortionViewing::structural(
                            portion,
                            protos::StructuralEnclosure::Braced,
                        ) else {
                            return Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Shape));
                        };
                        if parts.len() != #arity {
                            return Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Arity));
                        }
                        Ok(Self( #( #embodies, )* ))
                    }
                    fn portion(&self) -> protos::Portion {
                        datomic::PortionBuilding::structural(
                            "",
                            protos::StructuralEnclosure::Braced,
                            vec![ #( #portions, )* ],
                        )
                    }
                }
            })
        }
        TypeDeclaration::Enum { name, variants } => {
            let name_ident = ident(name)?;
            let embodies = variants
                .iter()
                .map(|v| variant_embody_arm(&name_ident, v))
                .collect::<Result<Vec<_>, _>>()?;
            let portions = variants
                .iter()
                .map(|v| variant_portion_arm(&name_ident, v))
                .collect::<Result<Vec<_>, _>>()?;
            let nested = variants
                .iter()
                .filter_map(|v| nested_datomic_impl(&name_ident, name, v).transpose())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                impl datomic::Datomic for #name_ident {
                    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
                        #( #embodies )*
                        Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Shape))
                    }
                    fn portion(&self) -> protos::Portion {
                        match self {
                            #( #portions )*
                        }
                    }
                }
                #( #nested )*
            })
        }
    }
}

fn variant_embody_arm(
    parent: &proc_macro2::Ident,
    variant: &Variant,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match variant {
        Variant::Unit(name) => {
            let name = ident(name)?;
            quote! {
                if datomic::PortionViewing::bare_symbol(portion) == Some(stringify!(#name)) {
                    return Ok(Self::#name);
                }
            }
        }
        Variant::Typed(name, ty) => {
            let variant_name = ident(name)?;
            let ty = type_expression_tokens(ty)?;
            quote! {
                if let Some(headed) = datomic::PortionViewing::headed(portion)
                    && headed.head.as_ref() == stringify!(#variant_name)
                    && headed.separator == protos::Separator::Period
                {
                    return Ok(Self::#variant_name(
                        <#ty as datomic::Datomic>::embody(&headed.body)?
                    ));
                }
            }
        }
        Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = ident(name)?;
            let inline_name = format_ident!("{}{}", parent, variant_name);
            quote! {
                if let Some(headed) = datomic::PortionViewing::headed(portion)
                    && headed.head.as_ref() == stringify!(#variant_name)
                    && headed.separator == protos::Separator::Period
                {
                    return Ok(Self::#variant_name(
                        <#inline_name as datomic::Datomic>::embody(&headed.body)?
                    ));
                }
            }
        }
    })
}

fn variant_portion_arm(
    _parent: &proc_macro2::Ident,
    variant: &Variant,
) -> Result<proc_macro2::TokenStream, Fault> {
    Ok(match variant {
        Variant::Unit(name) => {
            let name = ident(name)?;
            quote! {
                Self::#name => datomic::PortionBuilding::bare(stringify!(#name)),
            }
        }
        Variant::Typed(name, _) | Variant::InlineStruct(name, _) | Variant::InlineEnum(name, _) => {
            let variant_name = ident(name)?;
            quote! {
                Self::#variant_name(value) => datomic::PortionBuilding::headed(
                    stringify!(#variant_name),
                    protos::Separator::Period,
                    datomic::Datomic::portion(value),
                ),
            }
        }
    })
}

fn nested_datomic_impl(
    _parent: &proc_macro2::Ident,
    parent_name: &str,
    variant: &Variant,
) -> Result<Option<proc_macro2::TokenStream>, Fault> {
    match variant {
        Variant::InlineStruct(vname, fields) => {
            let inline_name = format!("{}{}", parent_name, vname);
            Ok(Some(datomic_impl_tokens(&TypeDeclaration::Struct {
                name: inline_name,
                fields: fields.clone(),
            })?))
        }
        Variant::InlineEnum(vname, inner) => {
            let inline_name = format!("{}{}", parent_name, vname);
            Ok(Some(datomic_impl_tokens(&TypeDeclaration::Enum {
                name: inline_name,
                variants: inner.clone(),
            })?))
        }
        _ => Ok(None),
    }
}

fn kind_declaration_tokens(kind: &KindDeclaration) -> Result<proc_macro2::TokenStream, Fault> {
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
            let ty = type_expression_tokens(&ac.ty)?;
            Ok(quote! { const #cname: #ty; })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let capability_tokens = capabilities
        .iter()
        .map(capability_method_tokens)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        pub trait #name #generic_params #supertrait_bounds {
            #( #associated_type_tokens )*
            #( #constant_tokens )*
            #( #capability_tokens )*
        }
    })
}

fn capability_method_tokens(cap: &Capability) -> Result<proc_macro2::TokenStream, Fault> {
    let name = format_ident!("{}", &cap.name);
    let return_type = type_expression_tokens(&cap.yield_type)?;

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
            let ty = type_expression_tokens(ty)?;
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
            let ty = type_expression_tokens(&r.ty)?;
            Ok(quote! { #variant_name(#ty) })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let embodies = references
        .iter()
        .map(|r| {
            let variant_name = ident(&r.name)?;
            let ty = type_expression_tokens(&r.ty)?;
            Ok(quote! {
                if let Some(headed) = datomic::PortionViewing::headed(portion)
                    && headed.head.as_ref() == stringify!(#variant_name)
                    && headed.separator == protos::Separator::Period
                {
                    return Ok(Self::#variant_name(
                        <#ty as datomic::Datomic>::embody(&headed.body)?
                    ));
                }
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    let portions = references
        .iter()
        .map(|r| {
            let variant_name = ident(&r.name)?;
            Ok(quote! {
                Self::#variant_name(value) => datomic::PortionBuilding::headed(
                    stringify!(#variant_name),
                    protos::Separator::Period,
                    datomic::Datomic::portion(value),
                ),
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;

    Ok(quote! {
        #derive pub enum #enum_name { #( #variants, )* }
        impl datomic::Datomic for #enum_name {
            fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
                #( #embodies )*
                Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Shape))
            }
            fn portion(&self) -> protos::Portion {
                match self { #( #portions )* }
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
    })
}

// ============================================================================
// Portion helpers
// ============================================================================

fn portion_headed(portion: &Portion) -> Option<&protos::Headed> {
    match portion {
        Portion::Headed(_, headed) => Some(headed),
        _ => None,
    }
}

fn portion_bare(portion: &Portion) -> Option<&str> {
    match portion {
        Portion::Bare(_, bare) => Some(bare.symbol.as_ref()),
        _ => None,
    }
}

fn portion_structural(portion: &Portion, enclosure: StructuralEnclosure) -> Option<&[Portion]> {
    match portion {
        Portion::Enclosed(_, enclosed) if enclosed.structural_enclosure() == Some(enclosure) => {
            enclosed.portions()
        }
        _ => None,
    }
}

fn portion_braced(portion: &Portion) -> Option<&[Portion]> {
    portion_structural(portion, StructuralEnclosure::Braced)
}

fn portion_bracketed(portion: &Portion) -> Option<&[Portion]> {
    portion_structural(portion, StructuralEnclosure::Bracketed)
}

fn portion_guillemets(portion: &Portion) -> Option<&[Portion]> {
    portion_structural(portion, StructuralEnclosure::Guillemets)
}

fn portion_angled(portion: &Portion) -> Option<&[Portion]> {
    portion_structural(portion, StructuralEnclosure::Angled)
}

fn bare_symbol(portion: &Portion) -> Result<&str, ()> {
    portion_bare(portion).ok_or(())
}

fn bare_integer(portion: &Portion) -> Result<i64, Fault> {
    let s = bare_symbol(portion).map_err(|()| fault_at(portion, Problem::Version))?;
    s.parse::<i64>()
        .map_err(|_| fault_at(portion, Problem::Version))
}

fn fault_at(portion: &Portion, problem: Problem) -> Fault {
    let extent: &Extent = portion.as_ref();
    Fault {
        extent: Extent {
            start: extent.start,
            end: extent.end,
        },
        problem,
    }
}

fn root_fault(source_len: usize) -> Fault {
    Fault {
        extent: Extent {
            start: 0,
            end: source_len,
        },
        problem: Problem::Root,
    }
}

fn emit_fault() -> Fault {
    Fault {
        extent: Extent { start: 0, end: 0 },
        problem: Problem::Emission,
    }
}

fn ident(name: &str) -> Result<proc_macro2::Ident, Fault> {
    Ok(format_ident!("{}", name))
}
