//! Ethos-zero: an Ethos File dialect over the Protos Portion pivot.
//!
//! This crate never reads Ethos characters itself.  Protos delineates text to
//! `Portion`; this reader matches that anatomy, and the emitter constructs a
//! `syn::File` through `quote` before formatting it.

use std::{
    fmt,
    io::Write,
    process::{Command, Stdio},
};

use protos::{Delineatable, EnclosedAnatomy, Portion, Separator, StructuralEnclosure, Text};
use quote::{ToTokens, format_ident, quote};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileFault {
    pub extent: Span,
    pub reason: FileFaultReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileFaultReason {
    Protos,
    Root,
    Header,
    Section,
    Declaration,
    TypeExpression,
    Import,
    UnresolvedImport,
    UnsupportedApplication,
    Rust,
}

impl fmt::Display for FileFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at {}..{}",
            self.reason, self.extent.start, self.extent.end
        )
    }
}

impl std::error::Error for FileFault {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: Version,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Channel {
    pub name: String,
    pub contract: i64,
    pub wire: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileLocation {
    pub directory: String,
    pub file: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportReference {
    Source {
        source: String,
        objects: Vec<String>,
    },
    Local {
        file: String,
        objects: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedImport {
    pub reference: ImportReference,
    pub location: FileLocation,
}

pub trait Manifest {
    fn resolve(&self, source: &str) -> Option<FileLocation>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Reference(String),
    Application {
        constructor: String,
        arguments: Vec<TypeExpression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantPayload {
    Unit,
    Type(TypeExpression),
    InlineStruct(Vec<Field>),
    InlineEnum(Vec<Variant>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: VariantPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeDeclaration {
    Alias {
        name: String,
        target: TypeExpression,
    },
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Capability {
    Yield(Vec<TypeExpression>),
    MutableYield(Vec<TypeExpression>),
    Standard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindDeclaration {
    pub name: String,
    pub constraints: Vec<String>,
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Association {
    pub ty: String,
    pub kinds: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceFile {
    pub header: Header,
    pub channel: Channel,
    pub imports: Vec<ResolvedImport>,
    pub input: Vec<TypeDeclaration>,
    pub output: Vec<TypeDeclaration>,
    pub refusal: Vec<TypeDeclaration>,
    pub stream: Vec<TypeDeclaration>,
    pub types: Vec<TypeDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaFile {
    pub header: Header,
    pub imports: Vec<ResolvedImport>,
    pub types: Vec<TypeDeclaration>,
    pub kinds: Vec<KindDeclaration>,
    pub associations: Vec<Association>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum File {
    Interface(InterfaceFile),
    Schema(SchemaFile),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceIndex {
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assembly {
    pub files: Vec<File>,
    pub imports: Vec<ResolvedImport>,
}

pub struct RustGeneration {
    pub file: File,
    pub syntax: syn::File,
}

pub struct FileReader<'manifest> {
    manifest: &'manifest dyn Manifest,
}

impl<'manifest> FileReader<'manifest> {
    pub fn new(manifest: &'manifest dyn Manifest) -> Self {
        Self { manifest }
    }

    pub fn read(&self, source: &str) -> Result<File, FileFault> {
        let text = Text::<()>::from(source);
        let delineation = text.delineate().map_err(|fault| FileFault {
            extent: Span {
                start: fault.extent.start,
                end: fault.extent.end,
            },
            reason: FileFaultReason::Protos,
        })?;
        let portions = delineation.portions;
        let Some((root, rest)) = portions.split_first() else {
            return Err(root_fault(source.len(), FileFaultReason::Root));
        };
        match headed(root, "Interface", Separator::Period) {
            Some(body) => self.interface(body, rest),
            None => match headed(root, "Schema", Separator::Period) {
                Some(body) => self.schema(body, rest),
                None => Err(fault(root, FileFaultReason::Root)),
            },
        }
    }

    fn interface(&self, version: &Portion, rest: &[Portion]) -> Result<File, FileFault> {
        let [channel, imports, body] = rest else {
            return Err(fault(version, FileFaultReason::Header));
        };
        let header = Header {
            version: version_of(version)?,
        };
        let channel_body = headed(channel, "Channel", Separator::Period)
            .ok_or_else(|| fault(channel, FileFaultReason::Header))?;
        let channel = channel_of(channel_body)?;
        let imports = self.imports(imports)?;
        let sections = braced_contents(body)?;
        if sections.len() != 5 {
            return Err(fault(body, FileFaultReason::Section));
        }
        Ok(File::Interface(InterfaceFile {
            header,
            channel,
            imports,
            input: declarations(&sections[0])?,
            output: declarations(&sections[1])?,
            refusal: declarations(&sections[2])?,
            stream: declarations(&sections[3])?,
            types: declarations(&sections[4])?,
        }))
    }

    fn schema(&self, version: &Portion, rest: &[Portion]) -> Result<File, FileFault> {
        let [imports, types, kinds, associations] = rest else {
            return Err(fault(version, FileFaultReason::Header));
        };
        Ok(File::Schema(SchemaFile {
            header: Header {
                version: version_of(version)?,
            },
            imports: self.imports(imports)?,
            types: declarations(types)?,
            kinds: kinds_of(kinds)?,
            associations: associations_of(associations)?,
        }))
    }

    fn imports(&self, portion: &Portion) -> Result<Vec<ResolvedImport>, FileFault> {
        bracket_contents(portion)?
            .iter()
            .map(|portion| {
                let reference = import_of(portion)?;
                let location = match &reference {
                    ImportReference::Source { source, .. } => self
                        .manifest
                        .resolve(source)
                        .ok_or_else(|| fault(portion, FileFaultReason::UnresolvedImport))?,
                    ImportReference::Local { file, .. } => FileLocation {
                        directory: String::new(),
                        file: file.clone(),
                    },
                };
                Ok(ResolvedImport {
                    reference,
                    location,
                })
            })
            .collect()
    }
}

pub struct RustEmitter;

impl Default for RustEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl RustEmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, file: &File) -> Result<String, FileFault> {
        let generation = self.generate(file)?;
        format_rust(generation.syntax.into_token_stream().to_string())
    }

    pub fn generate(&self, file: &File) -> Result<RustGeneration, FileFault> {
        let syntax = self.syntax(file)?;
        Ok(RustGeneration {
            file: file.clone(),
            syntax,
        })
    }

    pub fn syntax(&self, file: &File) -> Result<syn::File, FileFault> {
        let mut definitions = Vec::new();
        let mut section_roots = Vec::new();
        match file {
            File::Interface(interface) => {
                definitions.extend(interface.input.iter());
                definitions.extend(interface.output.iter());
                definitions.extend(interface.refusal.iter());
                definitions.extend(interface.stream.iter());
                definitions.extend(interface.types.iter());
                section_roots.push(section_root_tokens("Input", &interface.input)?);
                section_roots.push(section_root_tokens("Output", &interface.output)?);
                section_roots.push(section_root_tokens("Refusal", &interface.refusal)?);
                section_roots.push(section_root_tokens("Stream", &interface.stream)?);
            }
            File::Schema(schema) => definitions.extend(schema.types.iter()),
        }
        let mut tokens = quote! { #![allow(dead_code)] };
        for declaration in definitions {
            tokens.extend(declaration_tokens(declaration)?);
        }
        for section_root in section_roots {
            tokens.extend(section_root);
        }
        if let File::Schema(schema) = file {
            for kind in &schema.kinds {
                tokens.extend(kind_tokens(kind)?);
            }
            for association in &schema.associations {
                tokens.extend(association_tokens(association)?);
            }
        }
        syn::parse2(tokens).map_err(|_| root_fault(0, FileFaultReason::Declaration))
    }
}

fn format_rust(source: String) -> Result<String, FileFault> {
    let mut rustfmt = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| root_fault(0, FileFaultReason::Rust))?;
    rustfmt
        .stdin
        .as_mut()
        .ok_or_else(|| root_fault(0, FileFaultReason::Rust))?
        .write_all(source.as_bytes())
        .map_err(|_| root_fault(0, FileFaultReason::Rust))?;
    let output = rustfmt
        .wait_with_output()
        .map_err(|_| root_fault(0, FileFaultReason::Rust))?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|_| root_fault(0, FileFaultReason::Rust))
    } else {
        Err(root_fault(0, FileFaultReason::Rust))
    }
}

fn section_root_tokens(
    name: &str,
    declarations: &[TypeDeclaration],
) -> Result<proc_macro2::TokenStream, FileFault> {
    let root = identifier(name)?;
    let variants = declarations
        .iter()
        .map(|declaration| {
            let name = declaration_name(declaration);
            let variant = identifier(name)?;
            let ty = identifier(name)?;
            Ok(quote! { #variant(#ty) })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(quote! { #[derive(Clone, Debug, PartialEq, Eq)] pub enum #root { #( #variants, )* } })
}

fn declaration_name(declaration: &TypeDeclaration) -> &str {
    match declaration {
        TypeDeclaration::Alias { name, .. }
        | TypeDeclaration::Struct { name, .. }
        | TypeDeclaration::Enum { name, .. } => name,
    }
}

fn declaration_tokens(
    declaration: &TypeDeclaration,
) -> Result<proc_macro2::TokenStream, FileFault> {
    Ok(match declaration {
        TypeDeclaration::Alias { name, target } => {
            let name = identifier(name)?;
            let target = type_tokens(target)?;
            quote! { pub type #name = #target; }
        }
        TypeDeclaration::Struct { name, fields } => {
            let name = identifier(name)?;
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field_identifier(&field.name)?;
                    let ty = type_tokens(&field.ty)?;
                    Ok(quote! { pub #name: #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! { #[derive(Clone, Debug, PartialEq, Eq)] pub struct #name { #( #fields, )* } }
        }
        TypeDeclaration::Enum { name, variants } => {
            let name = identifier(name)?;
            let variants = variants
                .iter()
                .map(variant_tokens)
                .collect::<Result<Vec<_>, _>>()?;
            quote! { #[derive(Clone, Debug, PartialEq, Eq)] pub enum #name { #( #variants, )* } }
        }
    })
}

fn variant_tokens(variant: &Variant) -> Result<proc_macro2::TokenStream, FileFault> {
    let name = identifier(&variant.name)?;
    Ok(match &variant.payload {
        VariantPayload::Unit => quote! { #name },
        VariantPayload::Type(ty) => {
            let ty = type_tokens(ty)?;
            quote! { #name(#ty) }
        }
        VariantPayload::InlineStruct(fields) => {
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field_identifier(&field.name)?;
                    let ty = type_tokens(&field.ty)?;
                    Ok(quote! { #name: #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! { #name { #( #fields, )* } }
        }
        VariantPayload::InlineEnum(_) => {
            return Err(root_fault(0, FileFaultReason::UnsupportedApplication));
        }
    })
}

fn kind_tokens(kind: &KindDeclaration) -> Result<proc_macro2::TokenStream, FileFault> {
    let name = identifier(&kind.name)?;
    let constraints = kind
        .constraints
        .iter()
        .map(|constraint| identifier(constraint))
        .collect::<Result<Vec<_>, _>>()?;
    let capabilities = kind
        .capabilities
        .iter()
        .map(|capability| match capability {
            Capability::Yield(types) => {
                let [ty] = types.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let ty = type_tokens(ty)?;
                Ok(quote! { fn yield_(&self) -> #ty; })
            }
            Capability::MutableYield(types) => {
                let [ty] = types.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let ty = type_tokens(ty)?;
                Ok(quote! { fn yield_mut(&mut self) -> #ty; })
            }
            Capability::Standard => Ok(quote! {}),
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(if constraints.is_empty() {
        quote! { pub trait #name { #( #capabilities )* } }
    } else {
        quote! { pub trait #name: #( #constraints )+* { #( #capabilities )* } }
    })
}

fn association_tokens(association: &Association) -> Result<proc_macro2::TokenStream, FileFault> {
    let ty = identifier(&association.ty)?;
    let kinds = association
        .kinds
        .iter()
        .map(|kind| identifier(kind))
        .collect::<Result<Vec<_>, _>>()?;
    let implementations = kinds.iter().map(|kind| quote! { impl #kind for #ty {} });
    Ok(
        quote! { #( #implementations )* const _: fn() = || { fn carries<T: #( #kinds + )*>() {} carries::<#ty>(); }; },
    )
}

fn type_tokens(expression: &TypeExpression) -> Result<proc_macro2::TokenStream, FileFault> {
    match expression {
        TypeExpression::Reference(name) => Ok({
            let name = identifier(name)?;
            quote! { #name }
        }),
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Vector" && arguments.len() == 1 => {
            let inner = type_tokens(&arguments[0])?;
            Ok(quote! { Vec<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Result" && arguments.len() == 2 => {
            let ok = type_tokens(&arguments[0])?;
            let error = type_tokens(&arguments[1])?;
            Ok(quote! { Result<#ok, #error> })
        }
        TypeExpression::Application { .. } => {
            Err(root_fault(0, FileFaultReason::UnsupportedApplication))
        }
    }
}

fn identifier(value: &str) -> Result<proc_macro2::Ident, FileFault> {
    if syn::parse_str::<syn::Ident>(value).is_ok() {
        Ok(format_ident!("{}", value))
    } else {
        Err(root_fault(0, FileFaultReason::Declaration))
    }
}

fn field_identifier(value: &str) -> Result<proc_macro2::Ident, FileFault> {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_uppercase() && index != 0 {
            output.push('_');
        }
        output.extend(character.to_lowercase());
    }
    if matches!(output.as_str(), "type" | "ref" | "self" | "crate" | "super") {
        output.push('_');
    }
    identifier(&output)
}

fn version_of(portion: &Portion) -> Result<Version, FileFault> {
    let values = braced_contents(portion)?;
    let [major, minor, patch] = values else {
        return Err(fault(portion, FileFaultReason::Header));
    };
    Ok(Version {
        major: integer(major)?,
        minor: integer(minor)?,
        patch: integer(patch)?,
    })
}

fn channel_of(portion: &Portion) -> Result<Channel, FileFault> {
    let values = braced_contents(portion)?;
    let [name, contract, wire] = values else {
        return Err(fault(portion, FileFaultReason::Header));
    };
    Ok(Channel {
        name: bare(name)?.to_owned(),
        contract: integer(contract)?,
        wire: integer(wire)?,
    })
}

fn declarations(portion: &Portion) -> Result<Vec<TypeDeclaration>, FileFault> {
    let portions = bracket_contents(portion)?;
    let mut declarations = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let following = portions.get(index + 1);
        let (declaration, consumed) = declaration_of(&portions[index], following)?;
        declarations.push(declaration);
        index += 1 + usize::from(consumed);
    }
    Ok(declarations)
}

fn declaration_of(
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeDeclaration, bool), FileFault> {
    let (name, body) =
        any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    match structural(body) {
        Some((StructuralEnclosure::Braced, values)) => Ok((
            TypeDeclaration::Struct {
                name: name.to_owned(),
                fields: fields(values)?,
            },
            false,
        )),
        Some((StructuralEnclosure::Bracketed, values)) => Ok((
            TypeDeclaration::Enum {
                name: name.to_owned(),
                variants: variants(values)?,
            },
            false,
        )),
        _ => {
            let (target, consumed) = type_expression_with_following(body, following)?;
            Ok((
                TypeDeclaration::Alias {
                    name: name.to_owned(),
                    target,
                },
                consumed,
            ))
        }
    }
}

fn fields(portions: &[Portion]) -> Result<Vec<Field>, FileFault> {
    let mut fields = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let portion = &portions[index];
        if let Ok(name) = bare(portion) {
            fields.push(Field {
                name: name.to_owned(),
                ty: TypeExpression::Reference(name.to_owned()),
            });
            index += 1;
            continue;
        }
        let (name, body) =
            any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
        let (ty, consumed) = type_expression_with_following(body, portions.get(index + 1))?;
        fields.push(Field {
            name: name.to_owned(),
            ty,
        });
        index += 1 + usize::from(consumed);
    }
    Ok(fields)
}

fn variants(portions: &[Portion]) -> Result<Vec<Variant>, FileFault> {
    let mut parsed_variants = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let portion = &portions[index];
        let (variant, consumed) = match any_headed(portion) {
            Some((name, body)) => match structural(body) {
                Some((StructuralEnclosure::Braced, members)) => (
                    Variant {
                        name: name.to_owned(),
                        payload: VariantPayload::InlineStruct(fields(members)?),
                    },
                    false,
                ),
                Some((StructuralEnclosure::Bracketed, members)) => (
                    Variant {
                        name: name.to_owned(),
                        payload: VariantPayload::InlineEnum(variants(members)?),
                    },
                    false,
                ),
                _ => {
                    let (ty, consumed) =
                        type_expression_with_following(body, portions.get(index + 1))?;
                    (
                        Variant {
                            name: name.to_owned(),
                            payload: VariantPayload::Type(ty),
                        },
                        consumed,
                    )
                }
            },
            None => (
                Variant {
                    name: bare(portion)?.to_owned(),
                    payload: VariantPayload::Unit,
                },
                false,
            ),
        };
        parsed_variants.push(variant);
        index += 1 + usize::from(consumed);
    }
    Ok(parsed_variants)
}

fn kinds_of(portion: &Portion) -> Result<Vec<KindDeclaration>, FileFault> {
    bracket_contents(portion)?
        .iter()
        .map(|portion| {
            let (name, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            let values = bracket_contents(body)?;
            let mut constraints = Vec::new();
            let mut capabilities = Vec::new();
            for value in values {
                match headed_full(value) {
                    Some(("Yield", Separator::Period, body)) => capabilities.push(
                        Capability::Yield(type_expressions(bracket_contents(body)?)?),
                    ),
                    Some(("MutableYield", Separator::Exclamation, body)) => capabilities.push(
                        Capability::MutableYield(type_expressions(bracket_contents(body)?)?),
                    ),
                    Some(_) => return Err(fault(value, FileFaultReason::Declaration)),
                    None if bare(value)? == "Standard" => capabilities.push(Capability::Standard),
                    None => constraints.push(bare(value)?.to_owned()),
                }
            }
            Ok(KindDeclaration {
                name: name.to_owned(),
                constraints,
                capabilities,
            })
        })
        .collect()
}

fn associations_of(portion: &Portion) -> Result<Vec<Association>, FileFault> {
    bracket_contents(portion)?
        .iter()
        .map(|portion| {
            let (ty, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            Ok(Association {
                ty: ty.to_owned(),
                kinds: bracket_contents(body)?
                    .iter()
                    .map(|part| bare(part).map(str::to_owned))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}

fn type_expression(portion: &Portion) -> Result<TypeExpression, FileFault> {
    if let Ok(name) = bare(portion) {
        return Ok(TypeExpression::Reference(name.to_owned()));
    }
    Err(fault(portion, FileFaultReason::TypeExpression))
}

fn type_expression_with_following(
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeExpression, bool), FileFault> {
    let constructor = bare(portion)?;
    let Some(next) = following else {
        return Ok((TypeExpression::Reference(constructor.to_owned()), false));
    };
    let Some((StructuralEnclosure::Angled, arguments)) = structural(next) else {
        return Ok((TypeExpression::Reference(constructor.to_owned()), false));
    };
    Ok((
        TypeExpression::Application {
            constructor: constructor.to_owned(),
            arguments: arguments
                .iter()
                .map(type_expression)
                .collect::<Result<_, _>>()?,
        },
        true,
    ))
}

fn type_expressions(portions: &[Portion]) -> Result<Vec<TypeExpression>, FileFault> {
    let mut expressions = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let (expression, consumed) =
            type_expression_with_following(&portions[index], portions.get(index + 1))?;
        expressions.push(expression);
        index += 1 + usize::from(consumed);
    }
    Ok(expressions)
}

fn import_of(portion: &Portion) -> Result<ImportReference, FileFault> {
    if let Some((file, body)) = any_headed(portion)
        && file == "file"
    {
        return Ok(ImportReference::Local {
            file: file.to_owned(),
            objects: bracket_contents(body)?
                .iter()
                .map(|part| bare(part).map(str::to_owned))
                .collect::<Result<_, _>>()?,
        });
    }
    let (source, body) = headed_with_separator(portion, Separator::Colon)
        .ok_or_else(|| fault(portion, FileFaultReason::Import))?;
    if let Some((_file, members)) = any_headed(body) {
        return Ok(ImportReference::Source {
            source: source.to_owned(),
            objects: bracket_contents(members)?
                .iter()
                .map(|part| bare(part).map(str::to_owned))
                .collect::<Result<_, _>>()?,
        });
    }
    match structural(body) {
        Some((StructuralEnclosure::Bracketed, objects)) => Ok(ImportReference::Source {
            source: source.to_owned(),
            objects: objects
                .iter()
                .map(|part| bare(part).map(str::to_owned))
                .collect::<Result<_, _>>()?,
        }),
        _ => Ok(ImportReference::Source {
            source: source.to_owned(),
            objects: vec![bare(body)?.to_owned()],
        }),
    }
}

fn integer(portion: &Portion) -> Result<i64, FileFault> {
    bare(portion)?
        .parse()
        .map_err(|_| fault(portion, FileFaultReason::Header))
}
fn bare(portion: &Portion) -> Result<&str, FileFault> {
    match portion {
        Portion::Bare(_, bare) => Ok(bare.symbol.as_ref()),
        _ => Err(fault(portion, FileFaultReason::Declaration)),
    }
}
fn headed<'a>(portion: &'a Portion, name: &str, separator: Separator) -> Option<&'a Portion> {
    let (head, actual, body) = headed_full(portion)?;
    (head == name && actual == separator).then_some(body)
}
fn headed_with_separator(portion: &Portion, separator: Separator) -> Option<(&str, &Portion)> {
    let (head, actual, body) = headed_full(portion)?;
    (actual == separator).then_some((head, body))
}
fn any_headed(portion: &Portion) -> Option<(&str, &Portion)> {
    let (head, separator, body) = headed_full(portion)?;
    (separator == Separator::Period).then_some((head, body))
}
fn headed_full(portion: &Portion) -> Option<(&str, Separator, &Portion)> {
    match portion {
        Portion::Headed(_, headed) => {
            Some((headed.head.as_ref(), headed.separator, headed.body.as_ref()))
        }
        _ => None,
    }
}
fn structural(portion: &Portion) -> Option<(StructuralEnclosure, &[Portion])> {
    match portion {
        Portion::Enclosed(_, enclosed) => {
            Some((enclosed.structural_enclosure()?, enclosed.portions()?))
        }
        _ => None,
    }
}
fn bracket_contents(portion: &Portion) -> Result<&[Portion], FileFault> {
    structural(portion)
        .and_then(|(kind, values)| (kind == StructuralEnclosure::Bracketed).then_some(values))
        .ok_or_else(|| fault(portion, FileFaultReason::Section))
}
fn braced_contents(portion: &Portion) -> Result<&[Portion], FileFault> {
    structural(portion)
        .and_then(|(kind, values)| (kind == StructuralEnclosure::Braced).then_some(values))
        .ok_or_else(|| fault(portion, FileFaultReason::Header))
}
fn fault(portion: &Portion, reason: FileFaultReason) -> FileFault {
    let extent = portion.as_ref();
    FileFault {
        extent: Span {
            start: extent.start,
            end: extent.end,
        },
        reason,
    }
}
fn root_fault(end: usize, reason: FileFaultReason) -> FileFault {
    FileFault {
        extent: Span { start: 0, end },
        reason,
    }
}
