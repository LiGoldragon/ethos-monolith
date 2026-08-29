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
    Unit,
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
pub enum Visibility {
    Public,
    Crate,
    Private,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericParameter {
    pub name: String,
    pub default: Option<TypeExpression>,
    pub bounds: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub visibility: Visibility,
    pub name: String,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantPayload {
    Unit,
    Type(TypeExpression),
    Tuple(Vec<TypeExpression>),
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
        visibility: Visibility,
        name: String,
        generics: Vec<GenericParameter>,
        target: TypeExpression,
    },
    Struct {
        visibility: Visibility,
        name: String,
        generics: Vec<GenericParameter>,
        fields: Vec<Field>,
    },
    TupleStruct {
        visibility: Visibility,
        name: String,
        generics: Vec<GenericParameter>,
        fields: Vec<(Visibility, TypeExpression)>,
    },
    Enum {
        visibility: Visibility,
        name: String,
        generics: Vec<GenericParameter>,
        non_exhaustive: bool,
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
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParameter>,
    pub constraints: Vec<String>,
    pub associated: Vec<AssociatedType>,
    pub methods: Vec<Method>,
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Receiver {
    Shared,
    Mutable,
    Owned,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Method {
    pub name: String,
    pub generics: Vec<GenericParameter>,
    pub receiver: Receiver,
    pub inputs: Vec<Field>,
    pub output: TypeExpression,
    pub where_bounds: Vec<(String, Vec<String>)>,
    pub default: Option<DefaultBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefaultBody {
    Chain(Vec<DefaultTerm>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefaultTerm {
    SelfValue,
    Call {
        name: String,
        arguments: Vec<DefaultTerm>,
    },
    Path(Vec<String>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Association {
    pub ty: String,
    pub kinds: Vec<String>,
}

/// A named operation case refers to a declaration owned by the interface's type section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionReference {
    pub name: String,
    pub ty: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceFile {
    pub header: Header,
    pub channel: Channel,
    pub imports: Vec<ResolvedImport>,
    pub input: Vec<SectionReference>,
    pub output: Vec<SectionReference>,
    pub refusal: Vec<SectionReference>,
    pub stream: Vec<SectionReference>,
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
            input: section_references(&sections[0])?,
            output: section_references(&sections[1])?,
            refusal: section_references(&sections[2])?,
            stream: section_references(&sections[3])?,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustEmission {
    /// Emit consumer anatomy, including executable Datomic implementations.
    D3Consumer,
    /// Emit only the schema library's declarations, kinds, and association checks.
    SchemaLibrary,
    /// Emit a standalone rkyv Signal contract from an Interface file.
    WireContract,
}

pub struct RustEmitter {
    emission: RustEmission,
}

impl Default for RustEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl RustEmitter {
    pub fn new() -> Self {
        Self {
            emission: RustEmission::D3Consumer,
        }
    }

    pub fn schema_library() -> Self {
        Self {
            emission: RustEmission::SchemaLibrary,
        }
    }

    pub fn wire_contract() -> Self {
        Self {
            emission: RustEmission::WireContract,
        }
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
        if let (RustEmission::WireContract, File::Interface(interface)) = (self.emission, file) {
            return syn::parse2(wire_interface_tokens(interface)?)
                .map_err(|_| root_fault(0, FileFaultReason::Declaration));
        }
        let mut definitions = Vec::new();
        let mut section_roots: Vec<proc_macro2::TokenStream> = Vec::new();
        match file {
            File::Interface(interface) => {
                definitions.extend(interface.types.iter());
                section_roots.push(interface_root_tokens("Request", &interface.input, true)?);
                section_roots.push(interface_root_tokens("Reply", &interface.output, true)?);
                section_roots.push(interface_root_tokens("Refusal", &interface.refusal, true)?);
                section_roots.push(interface_root_tokens("Stream", &interface.stream, true)?);
            }
            File::Schema(schema) => definitions.extend(schema.types.iter()),
        }
        let mut tokens = quote! { #![allow(dead_code)] };
        for declaration in definitions {
            let datomic = is_datomic_file(file);
            tokens.extend(declaration_tokens(
                declaration,
                datomic,
                self.emission == RustEmission::SchemaLibrary,
            )?);
            if datomic
                && self.emission == RustEmission::D3Consumer
                && !matches!(declaration, TypeDeclaration::Alias { .. })
            {
                tokens.extend(datomic_anatomy_tokens(declaration)?);
            }
        }
        for section_root in section_roots {
            tokens.extend(section_root);
        }
        if let File::Schema(schema) = file {
            for kind in &schema.kinds {
                tokens.extend(kind_tokens(
                    kind,
                    is_datomic_file(file),
                    self.emission == RustEmission::SchemaLibrary,
                )?);
            }
            for association in &schema.associations {
                tokens.extend(association_tokens(association, &schema.types)?);
            }
        }
        syn::parse2(tokens).map_err(|_| root_fault(0, FileFaultReason::Declaration))
    }
}

fn wire_interface_tokens(interface: &InterfaceFile) -> Result<proc_macro2::TokenStream, FileFault> {
    let types = interface
        .types
        .iter()
        .map(wire_declaration_tokens)
        .collect::<Result<Vec<_>, _>>()?;
    let anatomies = interface
        .types
        .iter()
        .map(wire_datomic_tokens)
        .collect::<Result<Vec<_>, _>>()?;
    let request = wire_root_tokens("Request", &interface.input)?;
    let reply = wire_root_tokens("Reply", &interface.output)?;
    let refusal = wire_root_tokens("Refusal", &interface.refusal)?;
    let stream = wire_root_tokens("Stream", &interface.stream)?;
    let mut frame_bodies = vec![quote! { Request(Request) }, quote! { Reply(Reply) }];
    if !interface.refusal.is_empty() {
        frame_bodies.push(quote! { Refusal(Refusal) });
    }
    if !interface.stream.is_empty() {
        frame_bodies.push(quote! { Event(Stream) });
    }
    let major = u16::try_from(interface.header.version.major)
        .map_err(|_| root_fault(0, FileFaultReason::Header))?;
    let minor = u16::try_from(interface.header.version.minor)
        .map_err(|_| root_fault(0, FileFaultReason::Header))?;
    let patch = u16::try_from(interface.header.version.patch)
        .map_err(|_| root_fault(0, FileFaultReason::Header))?;
    let contract = u32::try_from(interface.channel.contract)
        .map_err(|_| root_fault(0, FileFaultReason::Header))?;
    let wire = u16::try_from(interface.channel.wire)
        .map_err(|_| root_fault(0, FileFaultReason::Header))?;
    Ok(quote! {
        use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)] pub struct ProtocolVersion { pub major: u16, pub minor: u16, pub patch: u16 }
        impl ProtocolVersion { pub const fn new(major: u16, minor: u16, patch: u16) -> Self { Self { major, minor, patch } } }
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)] pub struct ChannelContractId(pub u32);
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)] pub struct ChannelWireRevision(pub u16);
        pub const INTERFACE_VERSION: ProtocolVersion = ProtocolVersion::new(#major, #minor, #patch);
        pub const CHANNEL_CONTRACT_ID: ChannelContractId = ChannelContractId(#contract);
        pub const CHANNEL_WIRE_REVISION: ChannelWireRevision = ChannelWireRevision(#wire);
        pub const PROTOCOL_VERSION: ProtocolVersion = INTERFACE_VERSION;
        #( #types )* #( #anatomies )* #request #reply #refusal #stream
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] pub enum FrameBody { #( #frame_bodies, )* }
        #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] pub struct Frame { pub channel_contract_id: ChannelContractId, pub channel_wire_revision: ChannelWireRevision, pub protocol_version: ProtocolVersion, pub body: FrameBody }
    })
}

fn wire_datomic_tokens(
    declaration: &TypeDeclaration,
) -> Result<proc_macro2::TokenStream, FileFault> {
    match declaration {
        TypeDeclaration::Alias { name, target, .. } => {
            let name = identifier(name)?;
            match target {
                TypeExpression::Reference(value) if value == "String" => Ok(quote! {
                    impl datomic::Datomic for #name {
                        fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
                            Ok(Self(<datomic::DatomicString as datomic::Datomic>::embody(portion)?.as_ref().to_owned()))
                        }
                        fn portion(&self) -> protos::Portion {
                            datomic::DatomicString::try_from(self.0.clone()).map_or_else(
                                |_| datomic::PortionBuilding::bare("wire-invalid"),
                                |value| datomic::Datomic::portion(&value),
                            )
                        }
                    }
                }),
                TypeExpression::Reference(value) if value == "Integer" => Ok(quote! {
                    impl datomic::Datomic for #name {
                        fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> { Ok(Self(<i64 as datomic::Datomic>::embody(portion)?)) }
                        fn portion(&self) -> protos::Portion { datomic::Datomic::portion(&self.0) }
                    }
                }),
                _ => {
                    let target = wire_type_tokens(target)?;
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
            }
        }
        TypeDeclaration::Struct { .. } | TypeDeclaration::Enum { .. } => {
            datomic_anatomy_tokens(declaration)
        }
        TypeDeclaration::TupleStruct { .. } => Err(root_fault(0, FileFaultReason::Declaration)),
    }
}

fn wire_declaration_tokens(
    declaration: &TypeDeclaration,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let derive =
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] };
    match declaration {
        TypeDeclaration::Alias { name, target, .. } => {
            let name = identifier(name)?;
            if matches!(target, TypeExpression::Reference(value) if value == "String") {
                Ok(quote! {
                    #derive pub struct #name(String);

                    impl #name {
                        pub fn try_from_string(value: String) -> std::result::Result<Self, datomic::UnrepresentableString> {
                            datomic::DatomicString::try_from(value)
                                .map(|value| Self(value.as_ref().to_owned()))
                        }
                    }

                    impl std::convert::TryFrom<String> for #name {
                        type Error = datomic::UnrepresentableString;

                        fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
                            Self::try_from_string(value)
                        }
                    }

                    impl<'a> std::convert::TryFrom<&'a str> for #name {
                        type Error = datomic::UnrepresentableString;

                        fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
                            Self::try_from_string(value.to_owned())
                        }
                    }

                    impl AsRef<str> for #name {
                        fn as_ref(&self) -> &str {
                            &self.0
                        }
                    }
                })
            } else {
                let target = wire_type_tokens(target)?;
                Ok(quote! { #derive pub struct #name(pub #target); })
            }
        }
        TypeDeclaration::Struct { name, fields, .. } => {
            let name = identifier(name)?;
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field_identifier(&field.name)?;
                    let ty = wire_type_tokens(&field.ty)?;
                    Ok(quote! { pub #name: #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            Ok(quote! { #derive pub struct #name { #( #fields, )* } })
        }
        TypeDeclaration::TupleStruct { .. } => Err(root_fault(0, FileFaultReason::Declaration)),
        TypeDeclaration::Enum { name, variants, .. } => {
            let name = identifier(name)?;
            let variants = wire_variants(variants)?;
            Ok(quote! { #derive pub enum #name { #( #variants, )* } })
        }
    }
}

fn wire_root_tokens(
    name: &str,
    declarations: &[SectionReference],
) -> Result<proc_macro2::TokenStream, FileFault> {
    if declarations.is_empty() {
        return Ok(proc_macro2::TokenStream::new());
    }
    let name = identifier(name)?;
    let variants = declarations
        .iter()
        .map(|d| {
            let variant = identifier(&d.name)?;
            let ty = wire_type_tokens(&d.ty)?;
            Ok(quote! { #variant(#ty) })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(
        quote! { #[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)] pub enum #name { #( #variants, )* } },
    )
}
fn interface_root_tokens(
    name: &str,
    references: &[SectionReference],
    datomic: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let root = identifier(name)?;
    let variants = references
        .iter()
        .map(|reference| {
            let variant = identifier(&reference.name)?;
            let ty = type_tokens_with_datomic(&reference.ty, datomic, false)?;
            Ok(quote! { #variant(#ty) })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(quote! { pub enum #root { #( #variants, )* } })
}
fn wire_variants(variants: &[Variant]) -> Result<Vec<proc_macro2::TokenStream>, FileFault> {
    variants
        .iter()
        .map(|variant| {
            let name = identifier(&variant.name)?;
            Ok(match &variant.payload {
                VariantPayload::Unit => quote! { #name },
                VariantPayload::Type(ty) => {
                    let ty = wire_type_tokens(ty)?;
                    quote! { #name(#ty) }
                }
                VariantPayload::Tuple(types) => {
                    let types = types
                        .iter()
                        .map(wire_type_tokens)
                        .collect::<Result<Vec<_>, _>>()?;
                    quote! { #name( #( #types, )* ) }
                }
                _ => return Err(root_fault(0, FileFaultReason::Declaration)),
            })
        })
        .collect()
}
fn wire_type_tokens(expression: &TypeExpression) -> Result<proc_macro2::TokenStream, FileFault> {
    match expression {
        TypeExpression::Reference(name) if name == "String" => Ok(quote! { String }),
        TypeExpression::Reference(name) if name == "Integer" => Ok(quote! { i64 }),
        TypeExpression::Reference(_) => type_tokens_with_datomic(expression, false, false),
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Vector" && arguments.len() == 1 => {
            let inner = wire_type_tokens(&arguments[0])?;
            Ok(quote! { Vec<#inner> })
        }
        _ => type_tokens_with_datomic(expression, false, false),
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
                    let ty = type_tokens_with_datomic(&field.ty, true, false)?;
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
        TypeDeclaration::TupleStruct { .. } => {
            return Err(root_fault(0, FileFaultReason::Declaration));
        }
    };
    let nested = match declaration {
        TypeDeclaration::Enum { name, variants, .. } => nested_enum_anatomies(name, variants)?,
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
                    visibility: Visibility::Public,
                    name: format!("{parent}{variant_name}"),
                    generics: Vec::new(),
                    fields: fields.clone(),
                };
                tokens.extend(datomic_anatomy_tokens(&declaration)?);
            }
            VariantPayload::InlineEnum(members) => {
                let declaration = TypeDeclaration::Enum {
                    visibility: Visibility::Public,
                    name: format!("{parent}{variant_name}"),
                    generics: Vec::new(),
                    non_exhaustive: false,
                    variants: members.clone(),
                };
                tokens.extend(datomic_anatomy_tokens(&declaration)?);
            }
            VariantPayload::Unit | VariantPayload::Type(_) | VariantPayload::Tuple(_) => {}
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
                VariantPayload::Tuple(types) => {
                    let types = types
                        .iter()
                        .map(|ty| type_tokens_with_datomic(ty, true, false))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(quote! {
                        if let Some(headed) = datomic::PortionViewing::headed(portion)
                            && headed.head.as_ref() == stringify!(#variant_name)
                            && headed.separator == protos::Separator::Period
                        {
                            return Ok(Self::#variant_name( #( <#types as datomic::Datomic>::embody(&headed.body)?, )* ));
                        }
                    })
                }
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
                VariantPayload::Tuple(_) => Err(root_fault(0, FileFaultReason::Declaration)),
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
        fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
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
        VariantPayload::Type(ty) => type_tokens_with_datomic(ty, true, false),
        VariantPayload::InlineStruct(_) | VariantPayload::InlineEnum(_) => {
            let name = format_ident!("{}{}", parent, variant);
            Ok(quote! { #name })
        }
        VariantPayload::Unit | VariantPayload::Tuple(_) => {
            Err(root_fault(0, FileFaultReason::Declaration))
        }
    }
}

fn is_datomic_file(file: &File) -> bool {
    matches!(file, File::Schema(schema) if schema.kinds.iter().any(|kind| kind.name == "Datomic"))
        || matches!(file, File::Interface(_))
}

fn declaration_name(declaration: &TypeDeclaration) -> &str {
    match declaration {
        TypeDeclaration::Alias { name, .. }
        | TypeDeclaration::Struct { name, .. }
        | TypeDeclaration::TupleStruct { name, .. }
        | TypeDeclaration::Enum { name, .. } => name,
    }
}

fn visibility_tokens(visibility: &Visibility) -> proc_macro2::TokenStream {
    match visibility {
        Visibility::Public => quote! { pub },
        Visibility::Crate => quote! { pub(crate) },
        Visibility::Private => proc_macro2::TokenStream::new(),
    }
}

fn generic_tokens(
    generics: &[GenericParameter],
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    if generics.is_empty() {
        return Ok(proc_macro2::TokenStream::new());
    }
    let parameters = generics
        .iter()
        .map(|parameter| {
            let name = identifier(&parameter.name)?;
            let bounds = parameter
                .bounds
                .iter()
                .map(|bound| identifier(bound))
                .collect::<Result<Vec<_>, _>>()?;
            let default = parameter
                .default
                .as_ref()
                .map(|default| type_tokens_with_datomic(default, datomic, schema_library))
                .transpose()?;
            Ok(match (bounds.is_empty(), default) {
                (true, None) => quote! { #name },
                (false, None) => quote! { #name: #( #bounds )+* },
                (true, Some(default)) => quote! { #name = #default },
                (false, Some(default)) => quote! { #name: #( #bounds )+* = #default },
            })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    Ok(quote! { < #( #parameters ),* > })
}

fn declaration_tokens(
    declaration: &TypeDeclaration,
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    Ok(match declaration {
        TypeDeclaration::Alias {
            visibility,
            name,
            generics,
            target,
        } => {
            let name = identifier(name)?;
            let visibility = visibility_tokens(visibility);
            let generics = generic_tokens(generics, datomic, schema_library)?;
            let target = type_tokens_with_datomic(target, datomic, schema_library)?;
            quote! { #visibility type #name #generics = #target; }
        }
        TypeDeclaration::Struct {
            visibility,
            name,
            generics,
            fields,
        } => {
            let name = identifier(name)?;
            let visibility = visibility_tokens(visibility);
            let generics = generic_tokens(generics, datomic, schema_library)?;
            let fields = fields
                .iter()
                .map(|field| {
                    let name = field_identifier(&field.name)?;
                    let visibility = visibility_tokens(&field.visibility);
                    let ty = type_tokens_with_datomic(&field.ty, datomic, schema_library)?;
                    Ok(quote! { #visibility #name: #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! { #visibility struct #name #generics { #( #fields, )* } }
        }
        TypeDeclaration::TupleStruct {
            visibility,
            name,
            generics,
            fields,
        } => {
            let name = identifier(name)?;
            let visibility = visibility_tokens(visibility);
            let generics = generic_tokens(generics, datomic, schema_library)?;
            let fields = fields
                .iter()
                .map(|(visibility, ty)| {
                    let visibility = visibility_tokens(visibility);
                    let ty = type_tokens_with_datomic(ty, datomic, schema_library)?;
                    Ok(quote! { #visibility #ty })
                })
                .collect::<Result<Vec<_>, FileFault>>()?;
            quote! { #visibility struct #name #generics ( #( #fields, )* ); }
        }
        TypeDeclaration::Enum {
            visibility,
            name,
            generics,
            non_exhaustive,
            variants,
        } => {
            let name = identifier(name)?;
            let visibility = visibility_tokens(visibility);
            let generics = generic_tokens(generics, datomic, schema_library)?;
            enum_tokens(
                &visibility,
                &name,
                &generics,
                *non_exhaustive,
                variants,
                datomic,
                schema_library,
            )?
        }
    })
}

fn enum_tokens(
    visibility: &proc_macro2::TokenStream,
    parent: &proc_macro2::Ident,
    generics: &proc_macro2::TokenStream,
    non_exhaustive: bool,
    variants: &[Variant],
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let mut derived = proc_macro2::TokenStream::new();
    let mut emitted_variants = Vec::new();
    for variant in variants {
        let name = identifier(&variant.name)?;
        match &variant.payload {
            VariantPayload::Unit => emitted_variants.push(quote! { #name }),
            VariantPayload::Type(ty) => {
                let ty = type_tokens_with_datomic(ty, datomic, schema_library)?;
                emitted_variants.push(quote! { #name(#ty) });
            }
            VariantPayload::InlineStruct(fields) => {
                let derived_name = format_ident!("{}{}", parent, name);
                let fields = fields
                    .iter()
                    .map(|field| {
                        let name = field_identifier(&field.name)?;
                        let ty = type_tokens_with_datomic(&field.ty, datomic, schema_library)?;
                        Ok(quote! { pub #name: #ty })
                    })
                    .collect::<Result<Vec<_>, FileFault>>()?;
                derived.extend(quote! { pub struct #derived_name { #( #fields, )* } });
                emitted_variants.push(quote! { #name(#derived_name) });
            }
            VariantPayload::Tuple(types) => {
                let types = types
                    .iter()
                    .map(|ty| type_tokens_with_datomic(ty, datomic, schema_library))
                    .collect::<Result<Vec<_>, _>>()?;
                emitted_variants.push(quote! { #name( #( #types, )* ) });
            }
            VariantPayload::InlineEnum(members) => {
                let derived_name = format_ident!("{}{}", parent, name);
                derived.extend(enum_tokens(
                    &quote! { pub },
                    &derived_name,
                    &proc_macro2::TokenStream::new(),
                    false,
                    members,
                    datomic,
                    schema_library,
                )?);
                emitted_variants.push(quote! { #name(#derived_name) });
            }
        }
    }
    let non_exhaustive = non_exhaustive.then(|| quote! { #[non_exhaustive] });
    derived.extend(
        quote! { #non_exhaustive #visibility enum #parent #generics { #( #emitted_variants, )* } },
    );
    Ok(derived)
}

fn kind_tokens(
    kind: &KindDeclaration,
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let name = identifier(&kind.name)?;
    let visibility = visibility_tokens(&kind.visibility);
    let generics = generic_tokens(&kind.generics, false, false)?;
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
                let ty = type_tokens_with_datomic(ty, datomic, schema_library)?;
                Ok(quote! { fn #name(&self) -> #ty; })
            }
            Capability::Mutable { name, outputs } => {
                let [ty] = outputs.as_slice() else {
                    return Err(root_fault(0, FileFaultReason::Declaration));
                };
                let name = field_identifier(name)?;
                let ty = type_tokens_with_datomic(ty, datomic, schema_library)?;
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
                        let ty = type_tokens_with_datomic(ty, datomic, schema_library)?;
                        Ok(quote! { #name: #ty })
                    })
                    .collect::<Result<Vec<_>, FileFault>>()?;
                let output = type_tokens_with_datomic(output, datomic, schema_library)?;
                Ok(quote! { fn #name(&self, #( #inputs ),*) -> #output; })
            }
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    let associated = if !kind.associated.is_empty() {
        let associated = kind
            .associated
            .iter()
            .map(|associated| {
                let name = identifier(&associated.name)?;
                let bounds = associated
                    .bounds
                    .iter()
                    .map(|bound| identifier(bound))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(if bounds.is_empty() {
                    quote! { type #name; }
                } else {
                    quote! { type #name: #( #bounds )+*; }
                })
            })
            .collect::<Result<Vec<_>, FileFault>>()?;
        quote! { #( #associated )* }
    } else {
        match kind.name.as_str() {
            "Delineatable" => quote! { type Delineation; },
            "Embodiable" => quote! { type Embodied: Embodied; },
            _ => proc_macro2::TokenStream::new(),
        }
    };
    let capabilities = if !kind.methods.is_empty() {
        let methods = kind
            .methods
            .iter()
            .map(|method| method_tokens(method, datomic, schema_library))
            .collect::<Result<Vec<_>, _>>()?;
        quote! { #( #methods )* }
    } else if kind.name == "Delineatable" {
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
        quote! { #visibility trait #name #generics { #associated #capabilities } }
    } else {
        quote! { #visibility trait #name #generics: #( #constraints )+* { #associated #capabilities } }
    })
}

fn method_tokens(
    method: &Method,
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    let name = field_identifier(&method.name)?;
    let generics = generic_tokens(&method.generics, false, false)?;
    let receiver = match method.receiver {
        Receiver::Shared => quote! { &self },
        Receiver::Mutable => quote! { &mut self },
        Receiver::Owned => quote! { self },
        Receiver::None => proc_macro2::TokenStream::new(),
    };
    let inputs = method
        .inputs
        .iter()
        .map(|input| {
            let name = field_identifier(&input.name)?;
            let ty = type_tokens_with_datomic(&input.ty, datomic, schema_library)?;
            Ok(quote! { #name: #ty })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    let output = type_tokens_with_datomic(&method.output, datomic, schema_library)?;
    let where_bounds = method
        .where_bounds
        .iter()
        .map(|(subject, bounds)| {
            let subject = if subject == "Self" {
                quote! { Self }
            } else {
                let subject = identifier(subject)?;
                quote! { #subject }
            };
            let bounds = bounds
                .iter()
                .map(|bound| identifier(bound))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { #subject: #( #bounds )+* })
        })
        .collect::<Result<Vec<_>, FileFault>>()?;
    let inputs = if receiver.is_empty() {
        quote! { #( #inputs ),* }
    } else if inputs.is_empty() {
        receiver
    } else {
        quote! { #receiver, #( #inputs ),* }
    };
    let body = method
        .default
        .as_ref()
        .map(default_body_tokens)
        .transpose()?
        .map(|body| quote! { { #body } })
        .unwrap_or_else(|| quote! { ; });
    if where_bounds.is_empty() {
        Ok(quote! { fn #name #generics ( #inputs ) -> #output #body })
    } else {
        Ok(quote! { fn #name #generics ( #inputs ) -> #output where #( #where_bounds ),* #body })
    }
}

fn default_body_tokens(default: &DefaultBody) -> Result<proc_macro2::TokenStream, FileFault> {
    let DefaultBody::Chain(terms) = default;
    let Some((DefaultTerm::SelfValue, terms)) = terms.split_first() else {
        return Err(root_fault(0, FileFaultReason::Declaration));
    };
    let mut expression = quote! { self };
    for term in terms {
        let DefaultTerm::Call { name, arguments } = term else {
            return Err(root_fault(0, FileFaultReason::Declaration));
        };
        let name = field_identifier(name)?;
        let arguments = arguments
            .iter()
            .map(default_value_tokens)
            .collect::<Result<Vec<_>, _>>()?;
        expression = quote! { #expression.#name(#(#arguments),*) };
    }
    Ok(expression)
}

fn default_value_tokens(term: &DefaultTerm) -> Result<proc_macro2::TokenStream, FileFault> {
    match term {
        DefaultTerm::Path(segments) if !segments.is_empty() => {
            let segments = segments
                .iter()
                .map(|segment| identifier(segment))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { #(#segments)::* })
        }
        _ => Err(root_fault(0, FileFaultReason::Declaration)),
    }
}

fn association_tokens(
    association: &Association,
    declarations: &[TypeDeclaration],
) -> Result<proc_macro2::TokenStream, FileFault> {
    let ty = identifier(&association.ty)?;
    let kinds = association
        .kinds
        .iter()
        .map(|kind| identifier(kind))
        .collect::<Result<Vec<_>, _>>()?;
    let generics = declarations
        .iter()
        .find(|declaration| declaration_name(declaration) == association.ty)
        .map(|declaration| match declaration {
            TypeDeclaration::Alias { generics, .. }
            | TypeDeclaration::Struct { generics, .. }
            | TypeDeclaration::TupleStruct { generics, .. }
            | TypeDeclaration::Enum { generics, .. } => generics,
        })
        .ok_or_else(|| root_fault(0, FileFaultReason::Declaration))?;
    if generics.is_empty() {
        return Ok(quote! {
            const _: fn() = || {
                fn carries<T>() where T: #( #kinds + )* {}
                carries::<#ty>();
            };
        });
    }
    let parameters = generics
        .iter()
        .map(|generic| identifier(&generic.name))
        .collect::<Result<Vec<_>, _>>()?;
    let bounds = if association.kinds.iter().any(|kind| kind == "Embodiable") {
        quote! { : Embodied }
    } else {
        proc_macro2::TokenStream::new()
    };
    Ok(quote! {
        const _: fn() = || {
            fn carries<#( #parameters #bounds ),*>() {
                fn checked<U>() where U: #( #kinds + )* {}
                checked::<#ty<#( #parameters ),*>>();
            }
        };
    })
}

fn type_tokens_with_datomic(
    expression: &TypeExpression,
    datomic: bool,
    schema_library: bool,
) -> Result<proc_macro2::TokenStream, FileFault> {
    match expression {
        TypeExpression::Unit => Ok(quote! { () }),
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
                "StringSlice" => return Ok(quote! { str }),
                "PhantomData" => return Ok(quote! { std::marker::PhantomData }),
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
                    "Extent" => quote! { protos::Extent },
                    "Headed" => quote! { protos::Headed },
                    "OpaqueBoundary" => quote! { protos::OpaqueBoundary },
                    "StructuralEnclosure" => quote! { protos::StructuralEnclosure },
                    "Separator" => quote! { protos::Separator },
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
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { Vec<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Result" && arguments.len() == 2 => {
            let ok = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            let error = type_tokens_with_datomic(&arguments[1], datomic, schema_library)?;
            Ok(quote! { Result<#ok, #error> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if matches!(constructor.as_str(), "Option" | "Optional") && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { Option<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Text" && arguments.len() == 1 => {
            let target = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            if schema_library && !datomic {
                Ok(quote! { Text<#target> })
            } else {
                Ok(quote! { protos::Text<#target> })
            }
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Box" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { Box<#inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Phantom" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { std::marker::PhantomData<fn() -> #inner> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Borrowed" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { &#inner })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "BorrowedMut" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { &mut #inner })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Slice" && arguments.len() == 1 => {
            let inner = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { [#inner] })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Prospective" && arguments.len() == 1 => {
            let target = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            Ok(quote! { protos::Prospective<#target> })
        }
        TypeExpression::Application {
            constructor,
            arguments,
        } if constructor == "Map" && arguments.len() == 2 => {
            let key = type_tokens_with_datomic(&arguments[0], datomic, schema_library)?;
            let value = type_tokens_with_datomic(&arguments[1], datomic, schema_library)?;
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

fn section_references(portion: &Portion) -> Result<Vec<SectionReference>, FileFault> {
    bracket_contents(portion)?
        .iter()
        .map(|portion| {
            let (name, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            Ok(SectionReference {
                name: name.to_owned(),
                ty: type_expression(body)?,
            })
        })
        .collect()
}

fn declaration_of(
    portion: &Portion,
    following: Option<&Portion>,
) -> Result<(TypeDeclaration, bool), FileFault> {
    let (name, body) =
        any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    if let Some(contents) = headed(body, "Struct", Separator::Period).and_then(|body| {
        structural(body).and_then(|(enclosure, values)| {
            (enclosure == StructuralEnclosure::Braced).then_some(values)
        })
    }) {
        let (visibility, generics, values) = declaration_metadata(contents)?;
        return Ok((
            TypeDeclaration::Struct {
                visibility,
                name: name.to_owned(),
                generics,
                fields: fields(&values)?,
            },
            false,
        ));
    }
    if let Some(contents) = headed(body, "Tuple", Separator::Period).and_then(|body| {
        structural(body).and_then(|(enclosure, values)| {
            (enclosure == StructuralEnclosure::Bracketed).then_some(values)
        })
    }) {
        let (visibility, generics, values) = declaration_metadata(contents)?;
        let fields = values
            .iter()
            .map(tuple_field)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok((
            TypeDeclaration::TupleStruct {
                visibility,
                name: name.to_owned(),
                generics,
                fields,
            },
            false,
        ));
    }
    if let Some(contents) = headed(body, "Enum", Separator::Period).and_then(|body| {
        structural(body).and_then(|(enclosure, values)| {
            (enclosure == StructuralEnclosure::Bracketed).then_some(values)
        })
    }) {
        let (visibility, generics, mut values) = declaration_metadata(contents)?;
        let non_exhaustive = values.iter().any(|value| {
            headed(value, "NonExhaustive", Separator::Period)
                .is_some_and(|body| bare(body).is_ok_and(|value| value == "Yes"))
        });
        values.retain(|value| headed(value, "NonExhaustive", Separator::Period).is_none());
        return Ok((
            TypeDeclaration::Enum {
                visibility,
                name: name.to_owned(),
                generics,
                non_exhaustive,
                variants: variants(&values)?,
            },
            false,
        ));
    }
    if let Some(contents) = headed(body, "Alias", Separator::Period).and_then(|body| {
        structural(body).and_then(|(enclosure, values)| {
            (enclosure == StructuralEnclosure::Braced).then_some(values)
        })
    }) {
        let (visibility, generics, values) = declaration_metadata(contents)?;
        let [target, following @ ..] = values.as_slice() else {
            return Err(fault(body, FileFaultReason::Declaration));
        };
        let (label, target) =
            any_headed(target).ok_or_else(|| fault(target, FileFaultReason::Declaration))?;
        if label != "Target" {
            return Err(fault(target, FileFaultReason::Declaration));
        }
        let (target, _) = field_type_expression(target, following.first())?;
        return Ok((
            TypeDeclaration::Alias {
                visibility,
                name: name.to_owned(),
                generics,
                target,
            },
            false,
        ));
    }
    match structural(body) {
        Some((StructuralEnclosure::Braced, values)) => Ok((
            TypeDeclaration::Struct {
                visibility: Visibility::Public,
                name: name.to_owned(),
                generics: Vec::new(),
                fields: fields(values)?,
            },
            false,
        )),
        Some((StructuralEnclosure::Bracketed, values)) => Ok((
            TypeDeclaration::Enum {
                visibility: Visibility::Public,
                name: name.to_owned(),
                generics: Vec::new(),
                non_exhaustive: false,
                variants: variants(values)?,
            },
            false,
        )),
        Some((StructuralEnclosure::Guillemets, values)) => Ok((
            TypeDeclaration::Alias {
                visibility: Visibility::Public,
                name: name.to_owned(),
                generics: Vec::new(),
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
                    visibility: Visibility::Public,
                    name: name.to_owned(),
                    generics: Vec::new(),
                    target,
                },
                consumed,
            ))
        }
    }
}

fn declaration_metadata(
    values: &[Portion],
) -> Result<(Visibility, Vec<GenericParameter>, Vec<Portion>), FileFault> {
    let mut visibility = Visibility::Public;
    let mut generics = Vec::new();
    let mut remaining = Vec::new();
    for value in values {
        if let Some(body) = headed(value, "Visibility", Separator::Period) {
            visibility = visibility_of(body)?;
        } else if let Some(body) = headed(value, "Generics", Separator::Period) {
            let Some((StructuralEnclosure::Angled, values)) = structural(body) else {
                return Err(fault(value, FileFaultReason::Declaration));
            };
            generics = generic_parameters(values)?;
        } else {
            remaining.push(value.clone());
        }
    }
    Ok((visibility, generics, remaining))
}

fn visibility_of(portion: &Portion) -> Result<Visibility, FileFault> {
    match bare(portion)? {
        "Public" => Ok(Visibility::Public),
        "Crate" => Ok(Visibility::Crate),
        "Private" => Ok(Visibility::Private),
        _ => Err(fault(portion, FileFaultReason::Declaration)),
    }
}

fn generic_parameters(portions: &[Portion]) -> Result<Vec<GenericParameter>, FileFault> {
    portions
        .iter()
        .map(|portion| {
            if let Ok(name) = bare(portion) {
                return Ok(GenericParameter {
                    name: name.to_owned(),
                    default: None,
                    bounds: Vec::new(),
                });
            }
            let (name, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            let (default, bounds) = match structural(body) {
                Some((StructuralEnclosure::Bracketed, bounds)) => (
                    None,
                    bounds
                        .iter()
                        .map(bare)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                ),
                _ => (Some(type_expression(body)?), Vec::new()),
            };
            Ok(GenericParameter {
                name: name.to_owned(),
                default,
                bounds,
            })
        })
        .collect()
}

fn tuple_field(portion: &Portion) -> Result<(Visibility, TypeExpression), FileFault> {
    if let Some((name, body)) = any_headed(portion)
        && matches!(name, "Public" | "Crate" | "Private")
    {
        let visibility = match name {
            "Public" => Visibility::Public,
            "Crate" => Visibility::Crate,
            "Private" => Visibility::Private,
            _ => unreachable!(),
        };
        return Ok((visibility, type_expression(body)?));
    }
    Ok((Visibility::Private, type_expression(portion)?))
}

fn fields(portions: &[Portion]) -> Result<Vec<Field>, FileFault> {
    let mut fields = Vec::new();
    let mut index = 0;
    while index < portions.len() {
        let portion = &portions[index];
        if let Ok(name) = bare(portion) {
            fields.push(Field {
                visibility: Visibility::Public,
                name: name.to_owned(),
                ty: TypeExpression::Reference(name.to_owned()),
            });
            index += 1;
            continue;
        }
        let (name, body) =
            any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
        if matches!(name, "Public" | "Crate" | "Private") {
            let visibility = match name {
                "Public" => Visibility::Public,
                "Crate" => Visibility::Crate,
                "Private" => Visibility::Private,
                _ => unreachable!(),
            };
            let (name, body) =
                any_headed(body).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            let (ty, consumed) = field_type_expression(body, portions.get(index + 1))?;
            fields.push(Field {
                visibility,
                name: name.to_owned(),
                ty,
            });
            index += 1 + usize::from(consumed);
            continue;
        }
        let (ty, consumed) = field_type_expression(body, portions.get(index + 1))?;
        fields.push(Field {
            visibility: Visibility::Public,
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
            Some((name, body)) if headed(body, "Tuple", Separator::Period).is_some() => {
                let body = headed(body, "Tuple", Separator::Period).expect("checked Tuple head");
                let Some((StructuralEnclosure::Bracketed, values)) = structural(body) else {
                    return Err(fault(portion, FileFaultReason::Declaration));
                };
                (
                    Variant {
                        name: name.to_owned(),
                        payload: VariantPayload::Tuple(type_expressions(values)?),
                    },
                    false,
                )
            }
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
                    TypeExpression::Unit => {
                        return Err(fault(portion, FileFaultReason::Declaration));
                    }
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
            if let Some((StructuralEnclosure::Braced, values)) = structural(body)
                && values.iter().any(|value| {
                    any_headed(value).is_some_and(|(label, _)| {
                        [
                            "Visibility",
                            "Generics",
                            "Supertraits",
                            "Associated",
                            "Methods",
                        ]
                        .contains(&label)
                    })
                })
            {
                return declarative_kind(name, values);
            }
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
                visibility: Visibility::Public,
                name: name.to_owned(),
                generics: Vec::new(),
                constraints,
                associated: Vec::new(),
                methods: Vec::new(),
                capabilities,
            })
        })
        .collect()
}

fn declarative_kind(name: &str, values: &[Portion]) -> Result<KindDeclaration, FileFault> {
    let mut visibility = Visibility::Public;
    let mut generics = Vec::new();
    let mut constraints = Vec::new();
    let mut associated = Vec::new();
    let mut methods = Vec::new();
    for value in values {
        let Some((label, body)) = any_headed(value) else {
            return Err(fault(value, FileFaultReason::Declaration));
        };
        match label {
            "Visibility" => visibility = visibility_of(body)?,
            "Generics" => {
                let Some((StructuralEnclosure::Angled, values)) = structural(body) else {
                    return Err(fault(value, FileFaultReason::Declaration));
                };
                generics = generic_parameters(values)?;
            }
            "Supertraits" => {
                constraints = bracket_contents(body)?
                    .iter()
                    .map(bare)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            }
            "Associated" => {
                associated = associated_types(bracket_contents(body)?)?;
            }
            "Methods" => {
                methods = methods_of(bracket_contents(body)?)?;
            }
            _ => return Err(fault(value, FileFaultReason::Declaration)),
        }
    }
    Ok(KindDeclaration {
        visibility,
        name: name.to_owned(),
        generics,
        constraints,
        associated,
        methods,
        capabilities: Vec::new(),
    })
}

fn associated_types(portions: &[Portion]) -> Result<Vec<AssociatedType>, FileFault> {
    portions
        .iter()
        .map(|portion| {
            if let Ok(name) = bare(portion) {
                return Ok(AssociatedType {
                    name: name.to_owned(),
                    bounds: Vec::new(),
                });
            }
            let (name, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            let bounds = match structural(body) {
                Some((StructuralEnclosure::Bracketed, values)) => values
                    .iter()
                    .map(bare)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                _ => vec![bare(body)?.to_owned()],
            };
            Ok(AssociatedType {
                name: name.to_owned(),
                bounds,
            })
        })
        .collect()
}

fn methods_of(portions: &[Portion]) -> Result<Vec<Method>, FileFault> {
    portions
        .iter()
        .map(|portion| {
            let (name, body) =
                any_headed(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
            let Some((StructuralEnclosure::Braced, values)) = structural(body) else {
                return Err(fault(portion, FileFaultReason::Declaration));
            };
            let mut generics = Vec::new();
            let mut receiver = Receiver::None;
            let mut inputs = Vec::new();
            let mut output = None;
            let mut where_bounds = Vec::new();
            let mut default = None;
            let mut index = 0;
            while index < values.len() {
                let value = &values[index];
                let (label, body) =
                    any_headed(value).ok_or_else(|| fault(value, FileFaultReason::Declaration))?;
                match label {
                    "Generics" => {
                        let Some((StructuralEnclosure::Angled, values)) = structural(body) else {
                            return Err(fault(value, FileFaultReason::Declaration));
                        };
                        generics = generic_parameters(values)?;
                    }
                    "Receiver" => {
                        receiver = match bare(body)? {
                            "Shared" => Receiver::Shared,
                            "Mutable" => Receiver::Mutable,
                            "Owned" => Receiver::Owned,
                            "None" => Receiver::None,
                            _ => return Err(fault(value, FileFaultReason::Declaration)),
                        }
                    }
                    "Inputs" => inputs = fields(bracket_contents(body)?)?,
                    "Output" => {
                        let (ty, consumed) = field_type_expression(body, values.get(index + 1))?;
                        output = Some(ty);
                        index += usize::from(consumed);
                    }
                    "Where" => {
                        where_bounds = associated_types(bracket_contents(body)?)?
                            .into_iter()
                            .map(|associated| (associated.name, associated.bounds))
                            .collect()
                    }
                    "Default" => default = Some(default_body(body)?),
                    _ => return Err(fault(value, FileFaultReason::Declaration)),
                }
                index += 1;
            }
            Ok(Method {
                name: name.to_owned(),
                generics,
                receiver,
                inputs,
                output: output.ok_or_else(|| fault(portion, FileFaultReason::Declaration))?,
                where_bounds,
                default,
            })
        })
        .collect()
}

fn default_body(portion: &Portion) -> Result<DefaultBody, FileFault> {
    let (name, separator, body) =
        headed_full(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    if name != "Chain" || separator != Separator::Period {
        return Err(fault(portion, FileFaultReason::Declaration));
    }
    let Some((StructuralEnclosure::Bracketed, terms)) = structural(body) else {
        return Err(fault(portion, FileFaultReason::Declaration));
    };
    Ok(DefaultBody::Chain(
        terms
            .iter()
            .enumerate()
            .map(|(index, term)| default_term(term, index == 0))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn default_term(portion: &Portion, first: bool) -> Result<DefaultTerm, FileFault> {
    if let Ok(name) = bare(portion) {
        if first && name == "Self" {
            return Ok(DefaultTerm::SelfValue);
        }
        return Ok(DefaultTerm::Call {
            name: name.to_owned(),
            arguments: Vec::new(),
        });
    }
    let (name, separator, body) =
        headed_full(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    if separator != Separator::Period {
        return Err(fault(portion, FileFaultReason::Declaration));
    }
    let Some((StructuralEnclosure::Bracketed, arguments)) = structural(body) else {
        return Err(fault(portion, FileFaultReason::Declaration));
    };
    Ok(DefaultTerm::Call {
        name: name.to_owned(),
        arguments: arguments
            .iter()
            .map(default_value)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn default_value(portion: &Portion) -> Result<DefaultTerm, FileFault> {
    let (head, separator, tail) =
        headed_full(portion).ok_or_else(|| fault(portion, FileFaultReason::Declaration))?;
    if separator != Separator::Period {
        return Err(fault(portion, FileFaultReason::Declaration));
    }
    let tail = bare(tail)?;
    Ok(DefaultTerm::Path(vec![head.to_owned(), tail.to_owned()]))
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
        if name == "Unit" {
            return Ok(TypeExpression::Unit);
        }
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
            arguments: type_expressions(arguments)?,
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
        let (expression, consumed) = if any_headed(&portions[index]).is_some() {
            (type_expression(&portions[index])?, false)
        } else {
            type_expression_with_following(&portions[index], portions.get(index + 1))?
        };
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
