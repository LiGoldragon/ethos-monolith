//! Ethos-zero: an Ethos File dialect over the Protos Portion pivot.
//!
//! This crate never reads Ethos characters itself.  Protos delineates text to
//! `Portion`; this reader matches that anatomy, and the emitter constructs a
//! `syn::File` through `quote` before formatting it.

use std::{collections::BTreeMap, fmt, path::Path};

use protos::{
    Delineatable, EnclosedAnatomy, Extent, Portion, Separator, StructuralEnclosure, Text,
};
use quote::{ToTokens, format_ident, quote};

/// Handwritten Protos implementation items intentionally outside map ownership.
pub const PROTOS_ENGINE_ALGORITHMS: &[&str] = &[
    "Parser",
    "Printer",
    "TextHasher",
    "delimiter table",
    "normalization and extent computation",
];

/// Handwritten Datomic implementation items intentionally outside map ownership.
pub const DATOMIC_ENGINE_ALGORITHMS: &[&str] = &[
    "scalar Datomic implementations",
    "Vec, BTreeMap, and Option Datomic implementations",
    "PortionViewing",
    "PortionBuilding",
    "finite decimal and representable string validation",
];

#[derive(Clone, Eq, PartialEq)]
pub struct FileFault {
    pub extent: Extent,
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

impl fmt::Debug for FileFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FileFault")
            .field(
                "extent",
                &format_args!("{}..{}", self.extent.start, self.extent.end),
            )
            .field("reason", &self.reason)
            .finish()
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

/// A manifest embodied as a Datomic map from source name to relative file path.
pub struct DatomicManifest {
    sources: BTreeMap<datomic::DatomicString, datomic::DatomicString>,
}

impl DatomicManifest {
    pub fn embody(source: &str) -> Result<Self, datomic::Fault> {
        use datomic::TextEdge;
        let text = Text::<BTreeMap<datomic::DatomicString, datomic::DatomicString>>::from(source);
        Ok(Self {
            sources: text.embody()?,
        })
    }
}

impl Manifest for DatomicManifest {
    fn resolve(&self, source: &str) -> Option<FileLocation> {
        let key = datomic::DatomicString::try_from(source.to_owned()).ok()?;
        let value = self.sources.get(&key)?;
        let path = Path::new(value.as_ref());
        let file = path.file_name()?.to_string_lossy().into_owned();
        let directory = path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        Some(FileLocation { directory, file })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Reference(String),
    Associated {
        base: String,
        member: String,
    },
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
    Simple {
        name: String,
        outputs: Vec<TypeExpression>,
    },
    Mutable {
        name: String,
        outputs: Vec<TypeExpression>,
    },
    Standard {
        name: String,
        inputs: Vec<TypeExpression>,
        outputs: Vec<TypeExpression>,
    },
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
    pub sources: Vec<FileLocation>,
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
            extent: fault.extent,
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
        let types = headed(types, "Types", Separator::Period)
            .ok_or_else(|| fault(types, FileFaultReason::Section))?;
        let kinds = headed(kinds, "Kinds", Separator::Period)
            .ok_or_else(|| fault(kinds, FileFaultReason::Section))?;
        let associations = headed(associations, "Associations", Separator::Period)
            .ok_or_else(|| fault(associations, FileFaultReason::Section))?;
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
        Ok(generation.syntax.into_token_stream().to_string())
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
            let datomic = is_datomic_file(file);
            tokens.extend(declaration_tokens(declaration, datomic)?);
            if datomic && !matches!(declaration, TypeDeclaration::Alias { .. }) {
                tokens.extend(datomic_anatomy_tokens(declaration)?);
            }
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

fn datomic_anatomy_tokens(
    declaration: &TypeDeclaration,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let name = identifier(declaration_name(declaration))?;
    let body = match declaration {
        TypeDeclaration::Alias { .. } => return Err(root_fault(0, FileFaultReason::Declaration)),
        TypeDeclaration::Struct { fields, .. } => {
            let arity = fields.len();
            let embodies = fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let field_name = field_identifier(&field.name)?;
                    let ty = type_tokens_with_datomic(&field.ty, true)?;
                    Ok(quote! { #field_name: <#ty as datomic::Datomic>::embody(&parts[#index])? })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            let portions = fields
                .iter()
                .map(|field| {
                    let field_name = field_identifier(&field.name)?;
                    Ok(quote! { datomic::Datomic::portion(&self.#field_name) })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! {
                fn embody(portion: &protos::Portion) -> Result<Self, datomic::Fault> {
                    let Some(parts) = datomic::PortionViewing::structural(
                        portion,
                        protos::StructuralEnclosure::Braced,
                    ) else {
                        return Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Shape));
                    };
                    if parts.len() != #arity {
                        return Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Arity));
                    }
                    Ok(Self { #( #embodies, )* })
                }
                fn portion(&self) -> protos::Portion {
                    datomic::PortionBuilding::structural(
                        "",
                        protos::StructuralEnclosure::Braced,
                        vec![ #( #portions, )* ],
                    )
                }
            }
        }
        TypeDeclaration::Enum { variants, .. } => enum_anatomy_tokens(&name, variants)?,
    };
    let nested = match declaration {
        TypeDeclaration::Enum { name, variants } => nested_enum_anatomies(name, variants)?,
        _ => proc_macro2::TokenStream::new(),
    };
    Ok(quote! {
        impl datomic::Datomic for #name { #body }
        #nested
    })
}

fn nested_enum_anatomies(
    parent: &str,
    variants: &[Variant],
) -> Result<proc_macro2::TokenStream, FileFault> {
    let parent = identifier(parent)?;
    let mut tokens = proc_macro2::TokenStream::new();
    for variant in variants {
        let variant_name = identifier(&variant.name)?;
        match &variant.payload {
            VariantPayload::InlineStruct(fields) => {
                let declaration = TypeDeclaration::Struct {
                    name: format!("{parent}{variant_name}"),
                    fields: fields.clone(),
                };
                tokens.extend(datomic_anatomy_tokens(&declaration)?);
            }
            VariantPayload::InlineEnum(members) => {
                let declaration = TypeDeclaration::Enum {
                    name: format!("{parent}{variant_name}"),
                    variants: members.clone(),
                };
                tokens.extend(datomic_anatomy_tokens(&declaration)?);
            }
            VariantPayload::Unit | VariantPayload::Type(_) => {}
        }
    }
    Ok(tokens)
}

fn enum_anatomy_tokens(
    parent: &proc_macro2::Ident,
    variants: &[Variant],
) -> Result<proc_macro2::TokenStream, FileFault> {
    let embodiments = variants
        .iter()
        .map(|variant| {
            let variant_name = identifier(&variant.name)?;
            match &variant.payload {
                VariantPayload::Unit => Ok(quote! {
                    if datomic::PortionViewing::bare_symbol(portion) == Some(stringify!(#variant_name)) {
                        return Ok(Self::#variant_name);
                    }
                }),
                payload => {
                    let ty = variant_payload_type(parent, &variant_name, payload)?;
                    Ok(quote! {
                        if let Some(headed) = datomic::PortionViewing::headed(portion)
                            && headed.head.as_ref() == stringify!(#variant_name)
                            && headed.separator == protos::Separator::Period
                        {
                            return Ok(Self::#variant_name(<#ty as datomic::Datomic>::embody(&headed.body)?));
                        }
                    })
                }
            }
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    let portions = variants
        .iter()
        .map(|variant| {
            let variant_name = identifier(&variant.name)?;
            match &variant.payload {
                VariantPayload::Unit => Ok(quote! {
                    Self::#variant_name => datomic::PortionBuilding::bare(stringify!(#variant_name)),
                }),
                payload => {
                    let ty = variant_payload_type(parent, &variant_name, payload)?;
                    Ok(quote! {
                        Self::#variant_name(value) => datomic::PortionBuilding::headed(
                            stringify!(#variant_name),
                            protos::Separator::Period,
                            <#ty as datomic::Datomic>::portion(value),
                        ),
                    })
                }
            }
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(quote! {
        fn embody(portion: &protos::Portion) -> Result<Self, datomic::Fault> {
            #( #embodiments )*
            Err(datomic::PortionViewing::fault(portion, datomic::FaultProblem::Shape))
        }
        fn portion(&self) -> protos::Portion {
            match self { #( #portions )* }
        }
    })
}

fn variant_payload_type(
    parent: &proc_macro2::Ident,
    variant: &proc_macro2::Ident,
    payload: &VariantPayload,
) -> Result<proc_macro2::TokenStream, FileFault> {
    match payload {
        VariantPayload::Type(ty) => type_tokens_with_datomic(ty, true),
        VariantPayload::InlineStruct(_) | VariantPayload::InlineEnum(_) => {
            let name = format_ident!("{}{}", parent, variant);
            Ok(quote! { #name })
        }
        VariantPayload::Unit => Err(root_fault(0, FileFaultReason::Declaration)),
    }
}

fn is_datomic_file(file: &File) -> bool {
    matches!(file, File::Schema(schema) if schema.kinds.iter().any(|kind| kind.name == "Datomic"))
        || matches!(file, File::Interface(_))
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
    Ok(quote! { pub enum #root { #( #variants, )* } })
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
    datomic: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    Ok(match declaration {
        TypeDeclaration::Alias { name, target } => {
            let name = identifier(name)?;
            let target = type_tokens_with_datomic(target, datomic)?;
            quote! { pub type #name = #target; }
        }
        TypeDeclaration::Struct { name, fields } => {
            let name = identifier(name)?;
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field_identifier(&field.name)?;
                    let ty = type_tokens_with_datomic(&field.ty, datomic)?;
                    Ok(quote! { pub #name: #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! { pub struct #name { #( #fields, )* } }
        }
        TypeDeclaration::Enum { name, variants } => {
            let name = identifier(name)?;
            enum_tokens(&name, variants, datomic)?
        }
    })
}

fn enum_tokens(
    parent: &proc_macro2::Ident,
    variants: &[Variant],
    datomic: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let mut derived = proc_macro2::TokenStream::new();
    let mut emitted_variants = Vec::new();
    for variant in variants {
        let name = identifier(&variant.name)?;
        match &variant.payload {
            VariantPayload::Unit => emitted_variants.push(quote! { #name }),
            VariantPayload::Type(ty) => {
                let ty = type_tokens_with_datomic(ty, datomic)?;
                emitted_variants.push(quote! { #name(#ty) });
            }
            VariantPayload::InlineStruct(fields) => {
                let derived_name = format_ident!("{}{}", parent, name);
                let fields = fields
                    .iter()
                    .map(|field| {
                        let name = field_identifier(&field.name)?;
                        let ty = type_tokens_with_datomic(&field.ty, datomic)?;
                        Ok(quote! { pub #name: #ty })
                    })
                    .collect::<Result<Vec<_>, FileFault>>()?;
                derived.extend(quote! { pub struct #derived_name { #( #fields, )* } });
                emitted_variants.push(quote! { #name(#derived_name) });
            }
            VariantPayload::InlineEnum(members) => {
                let derived_name = format_ident!("{}{}", parent, name);
                derived.extend(enum_tokens(&derived_name, members, datomic)?);
                emitted_variants.push(quote! { #name(#derived_name) });
            }
        }
    }
    derived.extend(quote! { pub enum #parent { #( #emitted_variants, )* } });
    Ok(derived)
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
            Capability::Simple { name, outputs } => {
                let [ty] = outputs.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let name = field_identifier(name)?;
                let ty = type_tokens(ty)?;
                Ok(quote! { fn #name(&self) -> #ty; })
            }
            Capability::Mutable { name, outputs } => {
                let [ty] = outputs.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let name = field_identifier(name)?;
                let ty = type_tokens(ty)?;
                Ok(quote! { fn #name(&mut self) -> #ty; })
            }
            Capability::Standard {
                name,
                inputs,
                outputs,
            } => {
                let name = field_identifier(name)?;
                let [output] = outputs.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let inputs = inputs
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| {
                        let name = format_ident!("input_{index}");
                        let ty = type_tokens(ty)?;
                        Ok(quote! { #name: #ty })
                    })
                    .collect::<Result<Vec<_>, FileFault>>()?;
                let output = type_tokens(output)?;
                Ok(quote! { fn #name(&self, #( #inputs ),*) -> #output; })
            }
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    let associated = match kind.name.as_str() {
        "Delineatable" => quote! { type Delineation; },
        "Embodiable" => quote! { type Embodied: Embodied; },
        _ => proc_macro2::TokenStream::new(),
    };
    let capabilities = if kind.name == "Delineatable" {
        quote! { fn delineate(&self) -> Result<Self::Delineation, Fault>; }
    } else if kind.name == "Embodiable" {
        quote! { fn embody(&self) -> Result<Self::Embodied, Fault>; }
    } else if kind.name == "Embodied" {
        quote! { fn from_portion(portion: &Portion) -> Result<Self, Fault>; }
    } else if kind.name == "ShapeDefined" {
        quote! { fn matches(portion: &Portion) -> bool; }
    } else if kind.name == "DelineatedText" {
        quote! {
            fn delineation(&self) -> Option<&Delineation>;
            fn retag<U>(self) -> Text<U> where Self: Sized;
        }
    } else {
        quote! { #( #capabilities )* }
    };
    Ok(if constraints.is_empty() {
        quote! { pub trait #name { #associated #capabilities } }
    } else {
        quote! { pub trait #name: #( #constraints )+* { #associated #capabilities } }
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
    type_tokens_with_datomic(expression, false)
}

fn type_tokens_with_datomic(
    expression: &TypeExpression,
    datomic: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    match expression {
        TypeExpression::Reference(name) => Ok({
            if name == "Self" {
                return Ok(quote! { Self });
            }
            match name.as_str() {
                "Boolean" => return Ok(quote! { bool }),
                "Natural" | "ByteOffset" => return Ok(quote! { usize }),
                "UnsignedInteger" => return Ok(quote! { u64 }),
                "SignedInteger" | "Signed64" => return Ok(quote! { i64 }),
                "Float64" => return Ok(quote! { f64 }),
                _ => {}
            }
            if datomic {
                return Ok(match name.as_str() {
                    "String" => quote! { datomic::DatomicString },
                    "Integer" | "SignedInteger" | "Signed64" => quote! { i64 },
                    "Boolean" => quote! { bool },
                    "Decimal" | "FiniteDecimal" | "Float64" => quote! { datomic::FiniteDecimal },
                    "Portion" => quote! { protos::Portion },
                    "Text" => quote! { protos::Text },
                    _ => {
                        let name = identifier(name)?;
                        quote! { #name }
                    }
                });
            }
            let name = identifier(name)?;
            quote! { #name }
        }),
        TypeExpression::Associated { base, member } => {
            let base = if base == "Self" {
                quote! { Self }
            } else {
                let base = identifier(base)?;
                quote! { #base }
            };
            let member = identifier(member)?;
            Ok(quote! { #base :: #member })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Vector" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic)?;
            Ok(quote! { Vec<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Result" && arguments.len() == 2 => {
            let ok = type_tokens_with_datomic(&arguments[0], datomic)?;
            let error = type_tokens_with_datomic(&arguments[1], datomic)?;
            Ok(quote! { Result<#ok, #error> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if matches!(constructor.as_str(), "Option" | "Optional") && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic)?;
            Ok(quote! { Option<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Text" && arguments.len() == 1 => {
            let target = type_tokens_with_datomic(&arguments[0], datomic)?;
            Ok(quote! { protos::Text<#target> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Prospective" && arguments.len() == 1 => {
            let target = type_tokens_with_datomic(&arguments[0], datomic)?;
            Ok(quote! { protos::Prospective<#target> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Map" && arguments.len() == 2 => {
            let key = type_tokens_with_datomic(&arguments[0], datomic)?;
            let value = type_tokens_with_datomic(&arguments[1], datomic)?;
            Ok(quote! { std::collections::BTreeMap<#key, #value> })
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
    if matches!(
        output.as_str(),
        "type" | "ref" | "self" | "crate" | "super" | "yield"
    ) {
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
        Some((StructuralEnclosure::Guillemets, values)) => Ok((
            TypeDeclaration::Alias {
                name: name.to_owned(),
                target: TypeExpression::Application {
                    constructor: "Map".to_owned(),
                    arguments: fields(values)?.into_iter().map(|field| field.ty).collect(),
                },
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
        let (ty, consumed) = field_type_expression(body, portions.get(index + 1))?;
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
            None => {
                let (ty, consumed) =
                    type_expression_with_following(portion, portions.get(index + 1))?;
                match ty {
                    TypeExpression::Reference(name) => (
                        Variant {
                            name,
                            payload: VariantPayload::Unit,
                        },
                        consumed,
                    ),
                    TypeExpression::Application {
                        constructor,
                        arguments,
                    } => (
                        Variant {
                            name: constructor.clone(),
                            payload: VariantPayload::Type(TypeExpression::Application {
                                constructor,
                                arguments,
                            }),
                        },
                        consumed,
                    ),
                    TypeExpression::Associated { base, member } => (
                        Variant {
                            name: base,
                            payload: VariantPayload::Type(TypeExpression::Reference(member)),
                        },
                        consumed,
                    ),
                }
            }
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
            let (constraints, values) = match structural(body) {
                Some((StructuralEnclosure::Angled, identity)) => {
                    let mut constraints = Vec::new();
                    for constraint in identity {
                        match structural(constraint) {
                            Some((StructuralEnclosure::Bracketed, members)) => constraints.extend(
                                members
                                    .iter()
                                    .map(|member| bare(member).map(str::to_owned))
                                    .collect::<Result<Vec<_>, _>>()?,
                            ),
                            _ => constraints.push(bare(constraint)?.to_owned()),
                        }
                    }
                    (constraints, &[][..])
                }
                Some((StructuralEnclosure::Bracketed | StructuralEnclosure::Braced, values)) => {
                    (Vec::new(), values)
                }
                _ => return Err(fault(body, FileFaultReason::Declaration)),
            };
            let mut constraints = constraints;
            let mut capabilities = Vec::new();
            for value in values {
                if let Ok(constraint) = bare(value) {
                    constraints.push(constraint.to_owned());
                } else {
                    capabilities.push(capability_of(value)?);
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

fn capability_of(portion: &Portion) -> Result<Capability, FileFault> {
    let (name, separator, body) =
        headed_full(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    match (separator, structural(body)) {
        (Separator::Period, Some((StructuralEnclosure::Bracketed, outputs))) => {
            Ok(Capability::Simple {
                name: name.to_owned(),
                outputs: type_expressions(outputs)?,
            })
        }
        (Separator::Exclamation, Some((StructuralEnclosure::Bracketed, outputs))) => {
            Ok(Capability::Mutable {
                name: name.to_owned(),
                outputs: type_expressions(outputs)?,
            })
        }
        (Separator::Period, Some((StructuralEnclosure::Braced, form))) => {
            let [inputs, outputs] = form else {
                return Err(fault(body, FileFaultReason::Declaration));
            };
            Ok(Capability::Standard {
                name: name.to_owned(),
                inputs: type_expressions(bracket_contents(inputs)?)?,
                outputs: type_expressions(bracket_contents(outputs)?)?,
            })
        }
        _ => Err(fault(portion, FileFaultReason::Declaration)),
    }
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
    if let Some((StructuralEnclosure::Bracketed, members)) = structural(portion) {
        return Ok(TypeExpression::Application {
            constructor: "Vector".to_owned(),
            arguments: type_expressions(members)?,
        });
    }
    if let Some((StructuralEnclosure::Guillemets, members)) = structural(portion) {
        return Ok(TypeExpression::Application {
            constructor: "Map".to_owned(),
            arguments: type_expressions(members)?,
        });
    }
    if let Ok(name) = bare(portion) {
        return Ok(TypeExpression::Reference(name.to_owned()));
    }
    if let Some((base, Separator::Period, member)) = headed_full(portion) {
        if let Ok(member) = bare(member) {
            return Ok(TypeExpression::Associated {
                base: base.to_owned(),
                member: member.to_owned(),
            });
        }
        return Ok(TypeExpression::Reference(base.to_owned()));
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

fn field_type_expression(
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeExpression, bool), FileFault> {
    match structural(portion) {
        Some((StructuralEnclosure::Bracketed, members)) => Ok((
            TypeExpression::Application {
                constructor: "Vector".to_owned(),
                arguments: type_expressions(members)?,
            },
            false,
        )),
        Some((StructuralEnclosure::Guillemets, members)) => Ok((
            TypeExpression::Application {
                constructor: "Map".to_owned(),
                arguments: type_expressions(members)?,
            },
            false,
        )),
        _ if any_headed(portion).is_some() => Ok((type_expression(portion)?, false)),
        _ => type_expression_with_following(portion, following),
    }
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
    if let Some((file, body)) = any_headed(portion) {
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
        extent: Extent {
            start: extent.start,
            end: extent.end,
        },
        reason,
    }
}
fn root_fault(end: usize, reason: FileFaultReason) -> FileFault {
    FileFault {
        extent: Extent { start: 0, end },
        reason,
    }
}
